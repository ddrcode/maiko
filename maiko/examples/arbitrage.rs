use maiko::{Actor, Context, Topic};

#[derive(Clone, PartialEq, Eq, Hash)]
enum Exchange {
    Alpha,
    Beta,
}

#[derive(Clone)]
enum Side {
    Buy,
    Sell,
}

#[derive(Clone)]
struct Tick {
    ex: Exchange,
    price: f64,
    size: usize,
    side: Side,
}

impl Tick {
    fn new(ex: Exchange, price: f64, size: usize, side: Side) -> Self {
        Self {
            ex,
            price,
            size,
            side,
        }
    }
}

#[derive(Clone, Default)]
struct TopOfBook {
    bid: f64,
    bid_size: f64,
    ask: f64,
    ask_size: f64,
}

#[derive(maiko::Event, Clone)]
enum MarketEvent {
    AlphaTick {
        price: f64,
        quantity: u32,
    },
    BetaTick {
        price: String,
        side: char,
        count: u64,
    },
    MarketTick(Tick),
    Order(Tick),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum MarketTopic {
    RawData,
    NormalizedData,
    Order(Exchange),
}

impl Topic<MarketEvent> for MarketTopic {
    fn from_event(event: &MarketEvent) -> Self {
        use MarketEvent::*;
        match event {
            AlphaTick { .. } => MarketTopic::RawData,
            BetaTick { .. } => MarketTopic::RawData,
            MarketTick(_) => MarketTopic::NormalizedData,
            Order(tick) => MarketTopic::Order(tick.ex.clone()),
        }
    }
}

struct Ticker;
impl Actor for Ticker {
    type Event = MarketEvent;
}

struct Normalizer {
    ctx: Context<MarketEvent>,
}
impl Actor for Normalizer {
    type Event = MarketEvent;
    async fn handle_event(&mut self, envelope: &maiko::Envelope<Self::Event>) -> maiko::Result<()> {
        let tick = match &envelope.event() {
            MarketEvent::AlphaTick { price, quantity } => {
                Tick::new(Exchange::Alpha, *price, *quantity as usize, Side::Buy)
            }
            MarketEvent::BetaTick { price, side, count } => {
                let side_enum = match side {
                    'B' => Side::Buy,
                    'S' => Side::Sell,
                    _ => Side::Buy,
                };
                let price_f64 = price.parse::<f64>().unwrap();
                Tick::new(Exchange::Beta, price_f64, *count as usize, side_enum)
            }
            _ => return Ok(()),
        };
        self.ctx
            .send_child_event(MarketEvent::MarketTick(tick), envelope.meta())
            .await?;
        Ok(())
    }
}

struct Trader {
    ctx: Context<MarketEvent>,
    alpha_top: Option<TopOfBook>,
    beta_top: Option<TopOfBook>,
}
impl Trader {
    pub fn new(ctx: Context<MarketEvent>) -> Self {
        Self {
            ctx,
            alpha_top: None,
            beta_top: None,
        }
    }
    async fn process_tick(&mut self, tick: &Tick) {
        let top = match tick.ex {
            Exchange::Alpha => self.alpha_top.get_or_insert_default(),
            Exchange::Beta => self.beta_top.get_or_insert_default(),
        };
        match tick.side {
            Side::Buy => {
                top.bid = tick.price;
                top.bid_size += tick.size as f64;
            }
            Side::Sell => {
                top.ask = tick.price;
                top.ask_size += tick.size as f64;
            }
        }
    }

    fn try_arbitrage(&mut self) -> Option<Tick> {
        if let (Some(alpha), Some(beta)) = (&self.alpha_top, &self.beta_top) {
            if alpha.ask < beta.bid {
                Some(Tick::new(Exchange::Alpha, alpha.ask, 100, Side::Sell))
            } else if beta.ask < alpha.bid {
                Some(Tick::new(Exchange::Beta, beta.ask, 100, Side::Sell))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Actor for Trader {
    type Event = MarketEvent;
    async fn handle_event(&mut self, envelope: &maiko::Envelope<Self::Event>) -> maiko::Result<()> {
        if let MarketEvent::MarketTick(tick) = &envelope.event() {
            self.process_tick(tick).await;
            if let Some(order_tick) = self.try_arbitrage() {
                let other_side = Tick::new(
                    match order_tick.ex {
                        Exchange::Alpha => Exchange::Beta,
                        Exchange::Beta => Exchange::Alpha,
                    },
                    order_tick.price,
                    order_tick.size,
                    match order_tick.side {
                        Side::Buy => Side::Sell,
                        Side::Sell => Side::Buy,
                    },
                );
                self.ctx.send(MarketEvent::Order(other_side)).await?;
                self.ctx.send(MarketEvent::Order(order_tick)).await?;
            }
        }
        Ok(())
    }
}

struct Database;
impl Actor for Database {
    type Event = MarketEvent;
}

struct Telemetry;
impl Actor for Telemetry {
    type Event = MarketEvent;
}

#[tokio::main]
async fn main() -> maiko::Result {
    use Exchange::*;
    use MarketTopic::*;

    let mut sup = maiko::Supervisor::<MarketEvent, MarketTopic>::default();

    let alpha = sup.add_actor("AlphaTicker", |_ctx| Ticker, &[Order(Alpha)])?;
    sup.add_actor("BetaTicker", |_ctx| Ticker, &[Order(Beta)])?;
    let normalizer = sup.add_actor("Normalizer", |ctx| Normalizer { ctx }, &[RawData])?;
    let trader = sup.add_actor("Trader", Trader::new, &[NormalizedData])?;
    sup.add_actor("Database", |_ctx| Database, &[NormalizedData])?;
    let telemetry = sup.add_actor(
        "Telemetry",
        |_ctx| Telemetry,
        &[RawData, NormalizedData, Order(Alpha), Order(Beta)],
    )?;

    let mut test = sup.init_test_harness().await;
    sup.start().await?;
    test.start_recording().await;

    let id = test
        .send_as(
            &alpha,
            MarketEvent::AlphaTick {
                price: 100.0,
                quantity: 50,
            },
        )
        .await?;
    test.stop_recording().await;

    // Dump all recorded events
    test.dump();

    // Spy event sent as AlphaTicker
    let spy = test.event(id);
    assert!(spy.was_delivered_to(&normalizer));
    assert_eq!(2, spy.receivers_count());
    assert_eq!(3, spy.children().count());

    // Spy Normalizer actor
    let spy = test.actor(&normalizer);
    assert_eq!(
        1,
        spy.received_events_count(),
        "Normalizer should receive 1 event (from Alpha)"
    );
    assert_eq!(1, spy.sent_events_count(), "Normalizer should send 1 event");

    // Spy Telemetry actor
    let spy = test.actor(&telemetry);
    println!(
        "Receivers (actors that received events from Telemetry): {}",
        spy.receivers().join(", ")
    );
    println!(
        "Senders (actors that sent events to Telemetry): {}",
        spy.senders().join(", ")
    );
    assert_eq!(2, spy.received_events_count());
    assert_eq!(0, spy.sent_events_count());
    assert_eq!(0, spy.receivers_count());
    assert_eq!(2, spy.senders_count());

    let spy = test.topic(MarketTopic::NormalizedData);
    assert_eq!(3, spy.event_count());
    let query = spy.events();

    test.exit().await;
    sup.stop().await?;

    Ok(())
}
