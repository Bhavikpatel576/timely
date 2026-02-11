use rust_embed::Embed;

#[derive(Embed)]
#[folder = "dashboard/dist/"]
pub struct DashboardAssets;
