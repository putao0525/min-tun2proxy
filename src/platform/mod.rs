#[cfg(target_os = "macos")]
pub mod macos;


pub trait RouteTable: Sync + Sync {
    fn add_route(&self);
    fn del_route(&self);
    fn init_route_table(&self);
    fn free_route_table(&self);
}
