use zbus::Connection;

use mpris_player_info::proxies::StateServerProxy;

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let conn = Connection::session().await?;
            let proxy = StateServerProxy::new(&conn).await?;
            proxy.toggle().await?;
            Ok(())
        })
}
