pub mod app;

#[test]
fn just_test() {
    fn main() -> std::io::Result<()> {
        actix_web::rt::System::new(stringify!(main)).block_on(async move {
            {
                Ok(())
            }
        })
    }
}
