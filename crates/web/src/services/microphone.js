const MAX_DURATION_MS = 30000;
const SAMPLE_INTERVAL_MS = 50;
const BAR_COUNT = 40;

export class MicrophoneRecorder {
  constructor(onLevels, onStatus, onFinished, onError) {
    this.onLevels = onLevels;
    this.onStatus = onStatus;
    this.onFinished = onFinished;
    this.onError = onError;
    this.stream = null;
    this.context = null;
    this.analyser = null;
    this.recorder = null;
    this.chunks = [];
    this.objectUrl = null;
    this.frame = null;
    this.deadline = null;
    this.lastSample = 0;
    this.levels = Array(BAR_COUNT).fill(0);
    this.generation = 0;
    this.disposed = false;
    this.phase = "idle";
  }

  emitStatus(phase) {
    this.phase = phase;
    if (!this.disposed) this.onStatus(phase);
  }

  releaseCapture() {
    if (this.frame !== null) {
      cancelAnimationFrame(this.frame);
      this.frame = null;
    }
    if (this.deadline !== null) {
      clearTimeout(this.deadline);
      this.deadline = null;
    }
    if (this.stream) {
      this.stream.getTracks().forEach((track) => track.stop());
      this.stream = null;
    }
    if (this.context) {
      const context = this.context;
      this.context = null;
      void context.close().catch(() => {});
    }
    this.analyser = null;
  }

  discardAudio() {
    if (this.objectUrl !== null) {
      URL.revokeObjectURL(this.objectUrl);
      this.objectUrl = null;
    }
    this.chunks = [];
  }

  tick(time) {
    if (this.disposed || this.phase !== "recording") return;
    if (time - this.lastSample >= SAMPLE_INTERVAL_MS && this.analyser) {
      const samples = new Float32Array(this.analyser.fftSize);
      this.analyser.getFloatTimeDomainData(samples);
      let squares = 0;
      for (const value of samples) squares += value * value;
      const rms = Math.sqrt(squares / samples.length);
      this.levels.shift();
      this.levels.push(Math.min(1, rms * 6));
      this.onLevels(this.levels.slice());
      this.lastSample = time;
    }
    this.frame = requestAnimationFrame((nextTime) => this.tick(nextTime));
  }

  async start() {
    if (this.disposed || this.phase === "requesting" || this.phase === "recording" || this.phase === "processing") return;
    const generation = ++this.generation;
    this.discardAudio();
    this.levels.fill(0);
    this.onLevels(this.levels.slice());
    this.emitStatus("requesting");
    try {
      if (!globalThis.isSecureContext || !navigator.mediaDevices?.getUserMedia) {
        throw new Error("Microphone recording requires HTTPS or localhost and a supported browser.");
      }
      if (typeof MediaRecorder === "undefined" || typeof AudioContext === "undefined") {
        throw new Error("This browser does not support audio recording and live waveforms.");
      }
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      if (this.disposed || generation !== this.generation) {
        stream.getTracks().forEach((track) => track.stop());
        return;
      }
      this.stream = stream;
      this.context = new AudioContext();
      const source = this.context.createMediaStreamSource(stream);
      this.analyser = this.context.createAnalyser();
      this.analyser.fftSize = 1024;
      source.connect(this.analyser);
      await this.context.resume();
      if (this.disposed || generation !== this.generation) {
        this.releaseCapture();
        return;
      }
      const recorder = new MediaRecorder(stream);
      this.recorder = recorder;
      this.chunks = [];
      recorder.ondataavailable = (event) => {
        if (event.data?.size && generation === this.generation) this.chunks.push(event.data);
      };
      recorder.onerror = () => {
        if (generation !== this.generation || this.disposed) return;
        this.releaseCapture();
        this.emitStatus("error");
        this.onError("Recording failed. Please try again.");
      };
      recorder.onstop = () => {
        if (generation !== this.generation || this.disposed) return;
        const mime = recorder.mimeType || this.chunks[0]?.type || "audio/webm";
        const blob = new Blob(this.chunks, { type: mime });
        this.releaseCapture();
        if (blob.size === 0) {
          this.emitStatus("error");
          this.onError("No audio was recorded. Please try again.");
          return;
        }
        this.objectUrl = URL.createObjectURL(blob);
        this.emitStatus("ready");
        this.onFinished(this.objectUrl);
      };
      recorder.start(250);
      this.emitStatus("recording");
      this.lastSample = 0;
      this.frame = requestAnimationFrame((time) => this.tick(time));
      this.deadline = setTimeout(() => this.stop(), MAX_DURATION_MS);
    } catch (error) {
      if (generation !== this.generation || this.disposed) return;
      this.releaseCapture();
      this.emitStatus("error");
      const reason = error?.name === "NotAllowedError"
        ? "Microphone access was denied. Allow microphone access in browser settings to record."
        : error?.name === "NotFoundError"
          ? "No microphone was found. Connect a microphone and try again."
          : error?.message || "Microphone recording could not start.";
      this.onError(reason);
    }
  }

  stop() {
    if (this.disposed || this.phase !== "recording") return;
    this.emitStatus("processing");
    if (this.recorder?.state === "recording") this.recorder.stop();
    this.releaseCapture();
  }

  discard() {
    ++this.generation;
    if (this.recorder?.state === "recording") this.recorder.stop();
    this.recorder = null;
    this.releaseCapture();
    this.discardAudio();
    this.levels.fill(0);
    if (!this.disposed) {
      this.onLevels(this.levels.slice());
      this.emitStatus("idle");
    }
  }

  dispose() {
    this.disposed = true;
    this.discard();
  }
}

export function createMicrophoneRecorder(onLevels, onStatus, onFinished, onError) {
  return new MicrophoneRecorder(onLevels, onStatus, onFinished, onError);
}
