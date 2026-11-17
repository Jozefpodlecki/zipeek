use thirtyfour::WebDriver;

use super::{BrowserKind, ChromeDevTools, HarnessResult};

const MAX_WINDOW_DIMENSION: u32 = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewportBand {
    Mobile,
    Tablet,
    Desktop,
}

impl ViewportBand {
    fn for_width(width: u32) -> Self {
        match width {
            0..=639 => Self::Mobile,
            640..=1023 => Self::Tablet,
            _ => Self::Desktop,
        }
    }
}

pub struct BrowserViewport<'a> {
    driver: &'a WebDriver,
    browser: BrowserKind,
}

impl<'a> BrowserViewport<'a> {
    pub fn new(driver: &'a WebDriver) -> HarnessResult<Self> {
        Ok(Self::for_browser(driver, BrowserKind::from_environment()?))
    }

    pub fn for_browser(driver: &'a WebDriver, browser: BrowserKind) -> Self {
        Self { driver, browser }
    }

    pub async fn viewport(&self, width: u32, height: u32, mobile: bool) -> HarnessResult<()> {
        let observed = self.set_viewport(width, height, mobile).await?;
        if observed != (ViewportSize { width, height }) {
            return Err(format!(
                "{} cannot provide exact content viewport {width}x{height}; observed {}x{}",
                self.browser.as_str(),
                observed.width,
                observed.height
            )
            .into());
        }
        Ok(())
    }

    pub async fn responsive(
        &self,
        width: u32,
        height: u32,
        mobile: bool,
    ) -> HarnessResult<ViewportSize> {
        let observed = self.set_viewport(width, height, mobile).await?;
        if ViewportBand::for_width(observed.width) != ViewportBand::for_width(width) {
            return Err(format!(
                "{} cannot provide the requested responsive breakpoint at {width}px; observed {}x{}",
                self.browser.as_str(), observed.width, observed.height
            ).into());
        }
        if observed.width != width || observed.height != height {
            eprintln!(
                "{} constrained requested {width}x{height} viewport to {}x{}; testing the measured breakpoint",
                self.browser.as_str(),
                observed.width,
                observed.height
            );
        }
        Ok(observed)
    }

    async fn set_viewport(
        &self,
        width: u32,
        height: u32,
        mobile: bool,
    ) -> HarnessResult<ViewportSize> {
        validate_dimensions(width, height)?;
        match self.browser {
            BrowserKind::Chrome => {
                ChromeDevTools::new(self.driver)
                    .viewport(width, height, mobile)
                    .await?;
                self.measure().await
            }
            BrowserKind::Firefox => self.firefox_viewport(width, height).await,
        }
    }

    async fn measure(&self) -> HarnessResult<ViewportSize> {
        let inner = self
            .driver
            .execute(super::browser_scripts::READ_VIEWPORT_SIZE, vec![])
            .await?
            .convert::<Vec<u32>>()?;
        match inner.as_slice() {
            [width, height] if *width > 0 && *height > 0 => Ok(ViewportSize {
                width: *width,
                height: *height,
            }),
            _ => Err(format!("Invalid browser viewport probe: {inner:?}").into()),
        }
    }

    async fn firefox_viewport(&self, width: u32, height: u32) -> HarnessResult<ViewportSize> {
        let mut outer_width = width;
        let mut outer_height = height;
        let mut previous = None;
        for _ in 0..5 {
            self.driver
                .set_window_rect(0, 0, outer_width, outer_height)
                .await?;
            let inner = self.measure().await?;
            if inner.width == width && inner.height == height {
                return Ok(inner);
            }
            let new_width = adjust_outer_dimension(outer_width, width, inner.width);
            let new_height = adjust_outer_dimension(outer_height, height, inner.height);
            if previous == Some(inner) && new_width == outer_width && new_height == outer_height {
                return Ok(inner);
            }
            previous = Some(inner);
            outer_width = new_width;
            outer_height = new_height;
        }
        self.measure().await
    }
}

fn validate_dimensions(width: u32, height: u32) -> HarnessResult<()> {
    if width == 0 || height == 0 || width > MAX_WINDOW_DIMENSION || height > MAX_WINDOW_DIMENSION {
        return Err(format!("Invalid browser viewport {width}x{height}").into());
    }
    Ok(())
}

fn adjust_outer_dimension(outer: u32, requested: u32, actual: u32) -> u32 {
    let difference = i64::from(requested) - i64::from(actual);
    (i64::from(outer) + difference).clamp(1, i64::from(MAX_WINDOW_DIMENSION)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_dimensions_are_bounded() {
        assert!(validate_dimensions(390, 844).is_ok());
        assert!(validate_dimensions(0, 844).is_err());
        assert!(validate_dimensions(390, 0).is_err());
        assert!(validate_dimensions(8193, 844).is_err());
    }

    #[test]
    fn firefox_window_adjustment_compensates_for_browser_chrome() {
        assert_eq!(adjust_outer_dimension(390, 390, 375), 405);
        assert_eq!(adjust_outer_dimension(844, 844, 760), 928);
        assert_eq!(adjust_outer_dimension(390, 390, 390), 390);
        assert_eq!(adjust_outer_dimension(1, 390, 5000), 1);
    }

    #[test]
    fn responsive_breakpoints_are_explicit_and_measured() {
        assert_eq!(ViewportBand::for_width(320), ViewportBand::Mobile);
        assert_eq!(ViewportBand::for_width(390), ViewportBand::Mobile);
        assert_eq!(ViewportBand::for_width(500), ViewportBand::Mobile);
        assert_eq!(ViewportBand::for_width(639), ViewportBand::Mobile);
        assert_eq!(ViewportBand::for_width(640), ViewportBand::Tablet);
        assert_eq!(ViewportBand::for_width(1023), ViewportBand::Tablet);
        assert_eq!(ViewportBand::for_width(1024), ViewportBand::Desktop);
    }
}
