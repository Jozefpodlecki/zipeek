use thirtyfour::prelude::*;
use zipeek_e2e::page_objects::{WebDriverActions, WebElementQueryActions};
use zipeek_e2e::{Harness, HarnessResult, ScenarioContext, SuiteKind, page_objects};

async fn pinyin_lessons_are_instructional_not_self_certified(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/pinyin/initials").await?;
    let lesson = driver.query_single(page_objects::courses::LESSON).await?;
    zipeek_e2e::e2e_ensure!(lesson.text().await?.contains("Initial sounds"));
    zipeek_e2e::e2e_ensure!(lesson.text().await?.contains("Practise what you learned"));
    lesson
        .query_single("button")
        .await?;
    driver
        .query_single("[data-component='course-lesson'] button")
        .await?;
    driver.refresh().await?;
    zipeek_e2e::e2e_ensure!(
        driver.query_single(page_objects::courses::LESSON).await?.text().await?.contains("Initial sounds")
    );
    ctx.open_route("/courses/pinyin").await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-pinyin-region]").await?.len() == 6);
    Ok(())
}

async fn catalogue_lesson_links_open_authored_content(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/pinyin").await?;
    driver.query_single(page_objects::courses::PINYIN).await?;
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-pinyin-region]").await?.len() == 6);
    for (index, slug) in [
        "tones", "initials", "simple-finals", "nasal-finals", "syllable-building", "sound-contrasts",
    ].iter().enumerate() {
        ctx.open_route("/courses/pinyin").await?;
        driver.find_all_css("[data-pinyin-region]").await?[index].click().await?;
        driver.query_single(&format!("[data-pinyin-details] a[href*='/courses/pinyin/{slug}']")).await?;
    }
    Ok(())
}

async fn all_pinyin_lessons_remain_navigable(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    let lessons = [
        "tones", "tone-patterns", "initials", "initial-contrasts",
        "simple-finals", "finals", "nasal-finals", "spelling-review",
        "syllable-building", "connected-speech", "sound-contrasts", "comprehensive-review",
    ];
    for (index, slug) in lessons.iter().enumerate() {
        ctx.open_route(&format!("/courses/pinyin/{slug}")).await?;
        let lesson = driver.query_single(page_objects::courses::LESSON).await?;
        zipeek_e2e::e2e_ensure!(lesson.text().await?.contains(&format!("Module {} of 12", index + 1)));
        if let Some(next) = lessons.get(index + 1) {
            driver.query(By::XPath("//article[@data-component='course-lesson']//a[contains(., 'Next module')]")).single().await?.click().await?;
            zipeek_e2e::page_objects::browser::wait_for_hash_route(driver, &format!("/courses/pinyin/{next}")).await?;
        }
    }
    ctx.checkpoint("pinyin-twelve-modules-navigable").await?;
    Ok(())
}

async fn pinyin_written_check_persists_without_claiming_speech_mastery(
    ctx: ScenarioContext,
) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/pinyin/tones").await?;
    driver.query_single("[data-component='course-lesson'] button:not([data-listen])").await?;
    driver.query(By::XPath("//button[contains(., 'Practise what you learned')]")).single().await?.click().await?;
    let checks = driver.query_single("[data-pinyin-check]").await?;
    zipeek_e2e::e2e_ensure!(checks.text().await?.contains("written recognition"));
    let options = checks.find_all(By::Css("button[aria-pressed]")).await?;
    zipeek_e2e::e2e_ensure!(options.len() == 6);
    options[1].click().await?;
    options[3].click().await?;
    driver.query(By::XPath("//button[contains(., 'Check answers')]")).single().await?.click().await?;
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-pinyin-check]").await?.text().await?.contains("2 of 2 correct"));
    driver.refresh().await?;
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-pinyin-completed]").await?.is_displayed().await?);
    ctx.open_route("/courses/pinyin").await?;
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-pinyin-details]").await?.text().await?.contains("Check passed"));
    Ok(())
}

async fn unknown_course_lesson_exposes_recovery(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/pinyin/not-a-real-lesson").await?;
    let alert = driver
        .query_single("[data-component='main-content'] section[role='alert']")
        .await?;
    zipeek_e2e::e2e_ensure!(alert.text().await?.contains("Lesson not found"));
    alert
        .query_single("a[href*='/courses/pinyin']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-component='pinyin-course']")
        .await?;
    Ok(())
}

async fn mosaic_selection_and_guided_lesson(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    ctx.open_route("/courses/grammar").await?;
    let cells = driver.find_all_css("[data-mosaic-region]").await?;
    zipeek_e2e::e2e_ensure!(cells.len() == 100, "Grammar mosaic must have 100 regions");
    driver
        .query_single("[data-course-tracks] a[href*='/courses/vocabulary']")
        .await?
        .click()
        .await?;
    let vocabulary = driver.find_all_css("[data-vocabulary-region]").await?;
    zipeek_e2e::e2e_ensure!(
        vocabulary.len() == 100,
        "Vocabulary mosaic must have 100 regions"
    );
    driver
        .query_single("[data-vocabulary-region='vocabulary-1']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-vocabulary-details]")
            .await?
            .text()
            .await?
            .contains("Everyday actions"),
        "Vocabulary tile must show its authored details"
    );
    let save = driver.query_single("[data-course-create-word-set]").await?;
    save.click().await?;
    driver
        .query_single("[data-course-word-set-existing]")
        .await?;
    driver
        .query_single("[data-course-tracks] a[href$='/courses/grammar']")
        .await?
        .click()
        .await?;
    driver
        .query_single("[data-mosaic-region='grammar-2']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-course-details] a[href*='/courses/grammar/time-place']")
            .await?
            .is_displayed()
            .await?
    );
    driver
        .query_single("[data-mosaic-region='grammar-3']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-course-details] a[href*='/courses/grammar/possession-quantity']")
            .await?
            .is_displayed()
            .await?
    );
    driver
        .query_single("[data-mosaic-region='grammar-4']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-course-details] a[href*='/courses/grammar/descriptions']")
            .await?
            .is_displayed()
            .await?
    );
    driver
        .query_single("[data-mosaic-region='grammar-5']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-course-details] a[href*='/courses/grammar/wants-abilities']")
            .await?
            .is_displayed()
            .await?
    );
    driver
        .query_single("[data-mosaic-region='grammar-6']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-course-details] a[href*='/courses/grammar/topic/6']")
            .await?
            .is_displayed()
            .await?
    );
    driver
        .query_single("[data-mosaic-region='grammar-1']")
        .await?
        .click()
        .await?;
    zipeek_e2e::e2e_ensure!(
        driver.find_all_css("[data-grammar-play]").await?.is_empty(),
        "Grammar has one lesson entry, not a duplicate player",
    );
    driver.query_single("[data-course-details] a[href*='/courses/grammar/intro']").await?.click().await?;
    driver.query_single("[data-component='course-lesson-shell']").await?;
    driver.query_single("[data-grammar-lesson='grammar-1']").await?;
    for index in 0..4 {
        driver.query_single(&format!("[data-grammar-point-id] [lang='zh-Hans']")).await?;
        if index < 3 {
            driver.query_single("[data-grammar-next]").await?.click().await?;
        }
    }
    driver.query_single("[data-grammar-next]").await?.click().await?;
    for (index, choice) in [0, 1, 2, 0].into_iter().enumerate() {
        let selector = format!("[data-grammar-question='{index}'] button");
        let choices = driver.find_all_css(&selector).await?;
        zipeek_e2e::e2e_ensure_eq!(choices.len(), 3);
        choices[choice].click().await?;
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }
    driver.query_single("[data-guided-grammar-bank]").await?;
    ctx.checkpoint("grammar-guided-checks-staged").await?;
    let guided_orders: [&[&str]; 6] = [
        &["我", "学习", "中文"],
        &["她", "是", "老师"],
        &["我", "不", "喝", "茶"],
        &["你", "喜欢", "咖啡", "吗"],
        &["她", "不", "是", "老师"],
        &["你", "学习", "中文", "吗"],
    ];
    for (index, tiles) in guided_orders.iter().enumerate() {
        let activity = driver.query_single("[data-guided-grammar-bank]").await?;
        zipeek_e2e::e2e_ensure!(
            activity.attr("data-guided-activity").await?.is_some(),
            "Guided word-bank activity {index} was not displayed"
        );
        for tile in *tiles {
            let selector =
                format!("[data-guided-grammar-bank] button[aria-label='Add {tile} to sentence']");
            driver.query_single(&selector).await?.click().await?;
        }
        driver
            .query_single("[data-guided-check]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-guided-feedback]")
                .await?
                .text()
                .await?
                .contains("Correct"),
            "Guided activity {index} did not accept its authored answer"
        );
        driver
            .query_single("[data-guided-next]")
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-guided-bank-review]")
            .await?
            .text()
            .await?
            .contains("6 of 6"),
        "Guided bank did not complete all authored activities"
    );
    ctx.checkpoint("grammar-word-bank-review").await?;
    ctx.open_route("/courses/grammar/time-place").await?;
    driver
        .query_single("[data-grammar-lesson='grammar-2']")
        .await?;
    for _ in 0..5 {
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }
    for (index, choice) in [0, 1, 2, 0, 1].into_iter().enumerate() {
        let selector = format!("[data-grammar-question='{index}'] button");
        let choices = driver.find_all_css(&selector).await?;
        zipeek_e2e::e2e_ensure!(choices.len() == 3);
        choices[choice].click().await?;
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }

    let time_place_orders: [&[&str]; 6] = [
        &["我", "今天", "学习", "中文"],
        &["我", "在", "学校", "学习"],
        &["我", "今天", "在", "学校", "学习"],
        &["我", "明天", "去", "学校"],
        &["你", "在", "哪里", "学习"],
        &["我", "明天", "在", "学校", "学习"],
    ];
    for tiles in time_place_orders {
        for tile in tiles {
            let selector =
                format!("[data-guided-grammar-bank] button[aria-label='Add {tile} to sentence']");
            driver.query_single(&selector).await?.click().await?;
        }
        driver
            .query_single("[data-guided-check]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-guided-feedback]")
                .await?
                .text()
                .await?
                .contains("Correct"),
            "Course 2 guided answer was not accepted"
        );
        driver
            .query_single("[data-guided-next]")
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-guided-bank-review]")
            .await?
            .text()
            .await?
            .contains("6 of 6")
    );
    ctx.checkpoint("grammar-time-place-guided-review").await?;
    ctx.open_route("/courses/grammar/possession-quantity")
        .await?;
    driver
        .query_single("[data-grammar-lesson='grammar-3']")
        .await?;
    for _ in 0..5 {
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }
    for (index, choice) in [1, 1, 0, 2, 0].into_iter().enumerate() {
        let selector = format!("[data-grammar-question='{index}'] button");
        let choices = driver.find_all_css(&selector).await?;
        zipeek_e2e::e2e_ensure!(choices.len() == 3);
        choices[choice].click().await?;
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }

    let possession_orders: [&[&str]; 6] = [
        &["我", "有", "一", "本", "书"],
        &["我", "没有", "电脑"],
        &["我", "有", "三", "个", "苹果"],
        &["你", "有", "几", "本", "书"],
        &["这里", "有", "两", "个", "人"],
        &["你", "有", "多少", "个", "苹果"],
    ];
    for tiles in possession_orders {
        for tile in tiles {
            let selector =
                format!("[data-guided-grammar-bank] button[aria-label='Add {tile} to sentence']");
            driver.query_single(&selector).await?.click().await?;
        }
        driver
            .query_single("[data-guided-check]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-guided-feedback]")
                .await?
                .text()
                .await?
                .contains("Correct"),
            "Course 3 guided answer was not accepted"
        );
        driver
            .query_single("[data-guided-next]")
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-guided-bank-review]")
            .await?
            .text()
            .await?
            .contains("6 of 6")
    );
    ctx.checkpoint("grammar-possession-quantity-guided-review")
        .await?;
    ctx.open_route("/courses/grammar/descriptions").await?;
    driver
        .query_single("[data-grammar-lesson='grammar-4']")
        .await?;
    for _ in 0..5 {
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }
    for (index, choice) in [1, 0, 2, 1, 0].into_iter().enumerate() {
        let selector = format!("[data-grammar-question='{index}'] button");
        let choices = driver.find_all_css(&selector).await?;
        zipeek_e2e::e2e_ensure!(choices.len() == 3);
        choices[choice].click().await?;
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }

    let description_orders: [&[&str]; 6] = [
        &["他", "很", "忙"],
        &["这件衣服", "很", "漂亮"],
        &["今天", "不", "冷"],
        &["这杯茶", "太", "热", "了"],
        &["今天", "比", "昨天", "冷"],
        &["昨天", "比", "今天", "冷"],
    ];
    for tiles in description_orders {
        for tile in tiles {
            let selector =
                format!("[data-guided-grammar-bank] button[aria-label='Add {tile} to sentence']");
            driver.query_single(&selector).await?.click().await?;
        }
        driver
            .query_single("[data-guided-check]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-guided-feedback]")
                .await?
                .text()
                .await?
                .contains("Correct"),
            "Course 4 guided answer was not accepted"
        );
        driver
            .query_single("[data-guided-next]")
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-guided-bank-review]")
            .await?
            .text()
            .await?
            .contains("6 of 6")
    );
    ctx.checkpoint("grammar-descriptions-guided-review").await?;
    ctx.open_route("/courses/grammar/wants-abilities").await?;
    driver
        .query_single("[data-grammar-lesson='grammar-5']")
        .await?;
    for _ in 0..5 {
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }
    for (index, choice) in [1, 0, 2, 0, 1].into_iter().enumerate() {
        let selector = format!("[data-grammar-question='{index}'] button");
        let choices = driver.find_all_css(&selector).await?;
        zipeek_e2e::e2e_ensure!(choices.len() == 3);
        choices[choice].click().await?;
        driver.query_single("[data-grammar-next]").await?.click().await?;
    }

    let wants_orders: [&[&str]; 6] = [
        &["我", "想", "学", "中文"],
        &["我要", "一杯", "水"],
        &["她", "会", "说", "中文"],
        &["我", "今天", "不能", "去"],
        &["我", "可以", "坐", "这里", "吗"],
        &["我", "明天", "想", "学习"],
    ];
    for tiles in wants_orders {
        for tile in tiles {
            let selector =
                format!("[data-guided-grammar-bank] button[aria-label='Add {tile} to sentence']");
            driver.query_single(&selector).await?.click().await?;
        }
        driver
            .query_single("[data-guided-check]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-guided-feedback]")
                .await?
                .text()
                .await?
                .contains("Correct"),
            "Course 5 guided sentence was not accepted"
        );
        driver
            .query_single("[data-guided-next]")
            .await?
            .click()
            .await?;
    }
    zipeek_e2e::e2e_ensure!(
        driver
            .query_single("[data-guided-bank-review]")
            .await?
            .text()
            .await?
            .contains("6 of 6")
    );
    ctx.checkpoint("grammar-wants-abilities-guided-review")
        .await?;

    for (course, choices) in [
        ("grammar-1", &[1_usize, 2, 0, 2][..]),
        ("grammar-2", &[0_usize, 1, 2, 0, 1][..]),
        ("grammar-3", &[2_usize, 0, 1, 0, 2][..]),
        ("grammar-4", &[2_usize, 0, 1, 2, 1][..]),
        ("grammar-5", &[0_usize, 1, 2, 0, 1][..]),
    ] {
        ctx.open_route(&format!("/courses/grammar/assessment/{course}"))
            .await?;
        let assessment_root = driver.query_single("[data-grammar-assessment]").await?;
        zipeek_e2e::e2e_ensure!(
            assessment_root
                .attr("data-grammar-assessment")
                .await?
                .as_deref()
                == Some(course),
            "Navigating to {course} must display its own assessment"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-assessment-review]")
                .await?
                .is_empty()
        );
        for (index, choice) in choices.iter().enumerate() {
            let selector = format!("[data-assessment-item] button[aria-pressed]");
            let buttons = driver.find_all_css(&selector).await?;
            buttons[index * 3 + *choice].click().await?;
        }
        driver
            .query_single("button[type='submit']")
            .await?
            .click()
            .await?;
        let review = driver.query_single("[data-assessment-review]").await?;
        zipeek_e2e::e2e_ensure!(review.text().await?.contains(&format!(
            "{} / {}",
            choices.len(),
            choices.len()
        )));
        let point_review = driver
            .query_single("[data-assessment-point-review]")
            .await?;
        let point_text = point_review.text().await?;
        zipeek_e2e::e2e_ensure!(point_text.contains("Review by grammar point"));
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-assessment-review-needed]")
                .await?
                .text()
                .await?
                .contains("0 grammar points need review"),
            "A perfect score must not indicate missing grammar points"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css(
                    "[data-assessment-point-review] button[aria-label^='Go to grammar point']"
                )
                .await?
                .len()
                == choices.len(),
            "Each point review must offer keyboard-operable navigation"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-assessment-point-review] li")
                .await?
                .len()
                == choices.len(),
            "Expected one point review row for each {course} assessment item"
        );

        let construction_tiles: &[&str] = match course {
            "grammar-1" => &["我", "不", "学习", "中文"],
            "grammar-2" => &["我", "今天", "在", "学校", "学习"],
            "grammar-3" => &["我", "有", "三", "个", "苹果"],
            "grammar-4" => &["今天", "比", "昨天", "冷"],
            "grammar-5" => &["我", "可以", "坐", "这里", "吗"],
            _ => unreachable!("authored assessment route"),
        };
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-construction-review]")
                .await?
                .is_empty(),
            "Construction explanations must be withheld before submission"
        );
        for tile in construction_tiles {
            let selector = format!(
                "[data-construction-assessment] button[aria-label='Add {tile} to sentence']"
            );
            driver.query_single(&selector).await?.click().await?;
        }
        driver
            .query_single("[data-construction-submit]")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-construction-review]")
                .await?
                .text()
                .await?
                .contains("Correct sentence"),
            "Independent construction must grade the selected tile order"
        );
        ctx.checkpoint(&format!("{course}-construction-review"))
            .await?;
        driver
            .query_single("[data-construction-review] button")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-construction-review]")
                .await?
                .is_empty(),
            "Construction retry must conceal the answer"
        );
        ctx.checkpoint(&format!("{course}-assessment-draft"))
            .await?;

        driver
            .query_single("[data-assessment-review] button")
            .await?
            .click()
            .await?;
        zipeek_e2e::e2e_ensure!(
            driver
                .find_all_css("[data-assessment-review]")
                .await?
                .is_empty(),
            "Review remained visible after restarting {course}"
        );
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single("[data-assessment-progress]")
                .await?
                .text()
                .await?
                .contains("Answered 0"),
            "Restart did not clear answers for {course}"
        );
        zipeek_e2e::e2e_ensure!(
            !driver
                .query_single("button[type='submit']")
                .await?
                .is_enabled()
                .await?,
            "Draft can submit without answering every question after restart"
        );
        if course == "grammar-1" {
            for (index, choice) in choices.iter().enumerate() {
                let picked = if index == 0 {
                    (*choice + 1) % 3
                } else {
                    *choice
                };
                let buttons = driver
                    .find_all_css("[data-assessment-item] button[aria-pressed]")
                    .await?;
                buttons[index * 3 + picked].click().await?;
            }
            zipeek_e2e::e2e_ensure!(
                driver
                    .find_all_css("[data-assessment-review]")
                    .await?
                    .is_empty()
            );
            driver
                .query_single("button[type='submit']")
                .await?
                .click()
                .await?;
            let review = driver.query_single("[data-assessment-review]").await?;
            zipeek_e2e::e2e_ensure!(review.text().await?.contains("3 / 4"));
            let point_review = driver
                .query_single("[data-assessment-point-review]")
                .await?;
            zipeek_e2e::e2e_ensure!(
                point_review.text().await?.contains("Review needed"),
                "Incorrect choice must be highlighted in the per-point review"
            );
            zipeek_e2e::e2e_ensure!(
                driver
                    .query_single("[data-assessment-review-needed]")
                    .await?
                    .text()
                    .await?
                    .contains("1 grammar points need review"),
                "Wrong answers must appear in the review summary"
            );
            ctx.checkpoint("grammar-1-assessment-incorrect-review")
                .await?;
        }
    }
    ctx.open_route("/courses/grammar").await?;
    for id in [
        "grammar-1",
        "grammar-2",
        "grammar-3",
        "grammar-4",
        "grammar-5",
    ] {
        let selector = format!("[data-mosaic-region='{id}']");
        zipeek_e2e::e2e_ensure!(
            driver
                .query_single(&selector)
                .await?
                .attr("fill-opacity")
                .await?
                .as_deref()
                == Some(if id == "grammar-1" { "0.27" } else { "0.58" }),
            "Guided review must not claim mastery for {id}"
        );
    }
    Ok(())
}

async fn expanded_grammar_courses_are_authored_and_resumable(ctx: ScenarioContext) -> HarnessResult<()> {
    let driver = &ctx.driver;
    for (region, title) in [(6, "Aspect: completed events"), (12, "Possibility complements"), (20, "Sequence and complex actions")] {
        ctx.open_route(&format!("/courses/grammar/topic/{region}")).await?;
        let lesson = driver.query_single("[data-grammar-expanded]").await?;
        zipeek_e2e::e2e_ensure!(lesson.text().await?.contains(title));
        zipeek_e2e::e2e_ensure!(lesson.text().await?.contains("does not award verified mastery"));
        zipeek_e2e::e2e_ensure!(driver.query_single("[data-grammar-ai-example]").await?.is_displayed().await?);
        driver.query(By::XPath("//button[contains(., 'Continue')]")).single().await?.click().await?;
        zipeek_e2e::e2e_ensure!(driver.query_single("[data-grammar-step='1']").await?.is_displayed().await?);
        driver.refresh().await?;
        zipeek_e2e::e2e_ensure!(driver.query_single("[data-grammar-step='1']").await?.is_displayed().await?);
    }
    for (region, title) in [(21, "Serial-verb sequences"), (100, "Grammar consolidation and review")] {
        ctx.open_route(&format!("/courses/grammar/topic/{region}")).await?;
        let lesson = driver.query_single("[data-grammar-expanded]").await?;
        zipeek_e2e::e2e_ensure!(lesson.text().await?.contains(title));
        zipeek_e2e::e2e_ensure!(lesson.text().await?.contains("does not award verified mastery"));
    }
    ctx.open_route("/courses/grammar/topic/80").await?;
    for _ in 0..5 {
        driver.query(By::XPath("//button[contains(., 'Continue')]")).single().await?.click().await?;
    }
    let practice = driver.query_single("[data-grammar-expansion-practice]").await?;
    zipeek_e2e::e2e_ensure!(practice.text().await?.contains("Recall the Chinese sentence"));
    zipeek_e2e::e2e_ensure!(practice.text().await?.contains("Reveal sentence"));
    zipeek_e2e::e2e_ensure!(!driver.query(By::XPath("//button[contains(., 'Continue')]")).single().await?.is_enabled().await?);
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-grammar-expansion-practice][data-grammar-practice-phase='recall']").await?.is_displayed().await?);
    driver.query_single("[data-grammar-reveal]").await?.click().await?;
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-grammar-expansion-practice]").await?.text().await?.contains("Check the meaning"));
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-grammar-expansion-practice] button[data-listen]").await?.len() >= 5);
    zipeek_e2e::e2e_ensure!(driver.find_all_css("[data-grammar-choice]").await?.len() >= 5);
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-grammar-expansion-practice][data-grammar-practice-phase='recognition']").await?.is_displayed().await?);
    zipeek_e2e::e2e_ensure!(!driver.query(By::XPath("//button[contains(., 'Continue')]")).single().await?.is_enabled().await?);
    ctx.open_route("/courses/grammar").await?;
    driver.query_single("[data-mosaic-region='grammar-100']").await?.click().await?;
    zipeek_e2e::e2e_ensure!(driver.query_single("[data-course-details] a[href*='/courses/grammar/topic/100']").await?.is_displayed().await?);
    Ok(())
}

#[tokio::test]
async fn course_interactions() -> HarnessResult<()> {
    let mut harness = Harness::start(SuiteKind::Root).await?;
    harness
        .run(
            "pinyin_lessons_are_instructional_not_self_certified",
            pinyin_lessons_are_instructional_not_self_certified,
        )
        .await?;
    harness
        .run(
            "catalogue_lesson_links_open_authored_content",
            catalogue_lesson_links_open_authored_content,
        )
        .await?;
    harness
        .run(
            "all_pinyin_lessons_remain_navigable",
            all_pinyin_lessons_remain_navigable,
        )
        .await?;
    harness
        .run(
            "pinyin_written_check_persists_without_claiming_speech_mastery",
            pinyin_written_check_persists_without_claiming_speech_mastery,
        )
        .await?;
    harness
        .run(
            "unknown_course_lesson_recovery",
            unknown_course_lesson_exposes_recovery,
        )
        .await?;
    harness
        .run(
            "mosaic_selection_and_guided_lesson",
            mosaic_selection_and_guided_lesson,
        )
        .await?;
    harness
        .run(
            "expanded_grammar_courses_are_authored_and_resumable",
            expanded_grammar_courses_are_authored_and_resumable,
        )
        .await?;
    harness.finish().await
}
