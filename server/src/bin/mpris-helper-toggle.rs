use zbus::{dbus_proxy, Connection};

#[dbus_proxy(
    default_service = "com.github.robinmarchart.mprisutils",
    interface = "com.github.robinmarchart.mprisutils",
    default_path = "/com/github/robinmarchart/mprisutils"
)]
trait Server {
    fn toggle(&self) -> zbus::Result<bool>;
}

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let conn = Connection::session().await?;
            let proxy = ServerProxy::new(&conn).await?;
            proxy.toggle().await?;
            Ok(())
        })
}
