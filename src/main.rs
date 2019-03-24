use juniper::{graphql_object, EmptyMutation, RootNode};
use std::net::Ipv4Addr;
use std::str::FromStr;
use tokio::runtime::current_thread::Runtime;
use trust_dns::client::{ClientFuture, ClientHandle};
use trust_dns::rr::{DNSClass, Name, RData, RecordType};
use trust_dns::udp::UdpClientStream;
use warp::Filter;

#[derive(Default)]
pub struct Query;

graphql_object!(Query: Context | &self | {

    field dns(&executor) -> i32 {
        let mut runtime = Runtime::new().unwrap();

        let stream = UdpClientStream::new(([8,8,8,8], 53).into());

        let (bg, mut client) = ClientFuture::connect(stream);

        runtime.spawn(bg);

        let query = client.query(Name::from_str("www.example.com.").unwrap(), DNSClass::IN, RecordType::A);

        let response = runtime.block_on(query).unwrap();

        if let &RData::A(addr) = response.answers()[0].rdata() {
            assert_eq!(addr, Ipv4Addr::new(93, 184, 216, 34));
        }

        0
    }


});

pub struct Context {}
impl Context {}

impl juniper::Context for Context {}

type Schema = RootNode<'static, Query, EmptyMutation<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query::default(), EmptyMutation::<Context>::new())
}

fn main() {
    let state = warp::any().map(move || Context {});
    let api = warp::path("graphql").and(juniper_warp::make_graphql_filter(schema(), state.boxed()));
    warp::serve(api).run(([0, 0, 0, 0], 8080));
}
