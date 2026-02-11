use crate::db;
use crate::db::categories as db_categories;
use crate::error::Result;
use crate::web::router::build_router;

pub fn cmd_dashboard(port: u16) -> Result<()> {
    // Ensure builtin categories and rules are seeded
    let conn = db::open_default_db()?;
    db_categories::seed_builtin_categories(&conn)?;
    drop(conn);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let app = build_router();
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        eprintln!("Timely dashboard running at http://localhost:{}", port);

        // Auto-open browser on macOS
        #[cfg(target_os = "macos")]
        {
            let url = format!("http://localhost:{}", port);
            let _ = std::process::Command::new("open").arg(&url).spawn();
        }

        axum::serve(listener, app).await?;
        Ok(())
    })
}
