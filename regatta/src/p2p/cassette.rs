/// # Examples
/// # Panics
/// # Errors
/// # Safety
/// # Aborts
/// # Undefined Behavior
pub async fn read_local_recipes() -> super::Result<super::Recipes> {
    let content = tokio::fs::read(super::STORAGE_FILE_PATH).await?;
    let result = serde_json::from_slice(&content)?;
    Ok(result)
}
