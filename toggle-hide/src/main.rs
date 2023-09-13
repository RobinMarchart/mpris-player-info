use mpris_dbus::proxies::HideStateProxy;
use zbus::Connection;

fn main() -> zbus::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let conn = Connection::session().await?;
            let proxy = HideStateProxy::new(&conn).await?;
            proxy.toggle().await?;
            Ok(())
        })
}
