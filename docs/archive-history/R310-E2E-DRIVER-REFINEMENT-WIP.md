# E2E driver ergonomics - WIP after R309

The browser selector convenience API is implemented as `zipeek_e2e::page_objects::WebDriverActions`. Import the trait at the call site to use `driver.click(css).await?` or `driver.element(css).await?`. Both operations retain normal Thirtyfour `WebDriverResult` errors and `single()` lookup semantics.

Shared selectors are in `zipeek_e2e::page_objects::selectors::{practice,image_review}`. Recorder buttons assert their required event dispatch metadata; Image Review adds stable `data-image-review` attributes on its container, status, error and approval elements.

**Validation:** Node source contracts: 842 total, 729 passed and the same 113 pre-existing failures as R309. Four new focused source contracts passed. Rust compilation, rustfmt and browser E2E not executed because Cargo and Chrome were not available.

**Release status:** WIP only. Seven Rust paths have purposeful changes (including two new reusable E2E files); this does not satisfy the 20-substantive-Rust-file major release requirement. The existing R309 release notes are preserved unchanged.
