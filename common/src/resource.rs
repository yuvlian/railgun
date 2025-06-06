use rust_embed::Embed;

#[derive(Embed)]
#[folder = "include/resource/Config/LevelOutput/"]
pub struct LevelOutput;

#[derive(Embed)]
#[folder = "include/resource/ExcelOutput/"]
pub struct ExcelOutput;
