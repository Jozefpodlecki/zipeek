pub(crate) const MIXED_CLOUD_SIDES: &str = r#"
    const clouds = [...document.querySelectorAll('[data-cloud-pair-id][data-cloud-side]')];
    return ['left', 'right'].every(side =>
        clouds.some(cloud => cloud.dataset.cloudSide === side && cloud.dataset.cloudPrimary === 'true') &&
        clouds.some(cloud => cloud.dataset.cloudSide === side && cloud.dataset.cloudPrimary === 'false')
    );
"#;

pub(crate) const ACTIVE_SKY_SCENE: &str = r#"
    const scenes = [...document.querySelectorAll('[data-cloud-motion-controller] [data-cloud-scene]')];
    const active = scenes.filter(scene => scene.getAttribute('data-sky-active') === 'true');
    return scenes.length === 5 &&
        active.length === 1 &&
        ['0', '1', '2', '3', '4'].includes(active[0].getAttribute('data-cloud-scene'));
"#;

pub(crate) const MATCH_FADE_ACCESSIBLE: &str = r#"
    const ghosts = [...document.querySelectorAll('[data-cloud-ghost][data-cloud-success=true]')];
    return ghosts.length === 2 && ghosts.every(node => {
        const opacity = Number(getComputedStyle(node).opacity);
        return opacity >= 0 && opacity < 1 && node.getAttribute('aria-hidden') === 'true';
    });
"#;

pub(crate) const CLOUD_LABELS_FIT: &str = r#"
    const labels = [...document.querySelectorAll('[data-cloud-tile]:not([data-cloud-ghost]) [data-cloud-label]')];
    return labels.length === 6 && labels.every(label => {
        const cloud = label.closest('[data-cloud-tile]').getBoundingClientRect();
        const box = label.getBoundingClientRect();
        const inset = cloud.width * 0.10;
        return box.left >= cloud.left + inset &&
            box.right <= cloud.right - inset &&
            box.top >= cloud.top &&
            box.bottom <= cloud.bottom &&
            getComputedStyle(label).webkitLineClamp === '2';
    });
"#;

#[test]
fn cloud_match_script_contracts_are_not_empty() {
    for script in [
        MIXED_CLOUD_SIDES,
        ACTIVE_SKY_SCENE,
        MATCH_FADE_ACCESSIBLE,
        CLOUD_LABELS_FIT,
    ] {
        assert!(!script.trim().is_empty());
    }
}
