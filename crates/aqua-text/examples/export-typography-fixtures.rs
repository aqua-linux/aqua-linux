use aqua_text::TextService;

fn main() {
    let service = TextService::new().expect("embedded Aqua fonts should parse");
    print!("{}", service.typography_fixture_report());
}
