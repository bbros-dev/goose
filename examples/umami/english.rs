use swanling::prelude::*;

use crate::common;

use rand::seq::SliceRandom;

/// Load the front page in English and all static assets found on the page.
pub async fn front_page_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let swanling = user.get("/").await?;
    common::validate_and_load_static_assets(user, swanling, "Home").await?;

    Ok(())
}

/// Load recipe listing in English and all static assets found on the page.
pub async fn recipe_listing_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let swanling = user.get("/en/recipes/").await?;
    common::validate_and_load_static_assets(user, swanling, "Recipes").await?;

    Ok(())
}

/// Load a random recipe in English and all static assets found on the page.
pub async fn recipe_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let nodes = common::get_nodes(&common::ContentType::Recipe);
    let recipe = nodes.choose(&mut rand::thread_rng());
    let swanling = user.get(recipe.unwrap().url_en).await?;
    common::validate_and_load_static_assets(user, swanling, recipe.unwrap().title_en).await?;

    Ok(())
}

/// Load article listing in English and all static assets found on the page.
pub async fn article_listing_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let swanling = user.get("/en/articles/").await?;
    common::validate_and_load_static_assets(user, swanling, "Articles").await?;

    Ok(())
}

/// Load a random article in English and all static assets found on the page.
pub async fn article_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let nodes = common::get_nodes(&common::ContentType::Article);
    let article = nodes.choose(&mut rand::thread_rng());
    let swanling = user.get(article.unwrap().url_en).await?;
    common::validate_and_load_static_assets(user, swanling, article.unwrap().title_en).await?;

    Ok(())
}

/// Load a random basic page in English and all static assets found on the page.
pub async fn basic_page_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let nodes = common::get_nodes(&common::ContentType::BasicPage);
    let page = nodes.choose(&mut rand::thread_rng());
    let swanling = user.get(page.unwrap().url_en).await?;
    common::validate_and_load_static_assets(user, swanling, page.unwrap().title_en).await?;

    Ok(())
}

/// Load a random node by nid in English and all static assets found on the page.
pub async fn page_by_nid(user: &SwanlingUser) -> SwanlingTaskResult {
    // Randomly select a content type.
    let content_types = vec![
        common::ContentType::Article,
        common::ContentType::BasicPage,
        common::ContentType::Recipe,
    ];
    let content_type = content_types.choose(&mut rand::thread_rng());
    // Then randomly select a node of this content type.
    let nodes = common::get_nodes(&content_type.unwrap());
    let page = nodes.choose(&mut rand::thread_rng());
    // Load the page by nid instead of by URL.
    let swanling = user
        .get(&("/node/".to_string() + &page.unwrap().nid.to_string()))
        .await?;
    common::validate_and_load_static_assets(user, swanling, page.unwrap().title_en).await?;

    Ok(())
}

/// Anonymously load the contact form in English and POST feedback.
pub async fn anonymous_contact_form_en(user: &SwanlingUser) -> SwanlingTaskResult {
    common::anonymous_contact_form(user, true).await?;

    Ok(())
}

// Pick a random word from the title of a random node and perform a search in English.
pub async fn search_en(user: &SwanlingUser) -> SwanlingTaskResult {
    common::search(user, true).await?;

    Ok(())
}

/// Load category listing by a random term in English and all static assets found on the page.
pub async fn term_listing_en(user: &SwanlingUser) -> SwanlingTaskResult {
    let terms = common::get_terms();
    let term = terms.choose(&mut rand::thread_rng());
    let swanling = user.get(term.unwrap().url_en).await?;
    common::validate_and_load_static_assets(user, swanling, term.unwrap().title_en).await?;

    Ok(())
}
