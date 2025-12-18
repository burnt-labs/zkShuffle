use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Uint256};
use once_cell::sync::Lazy;

use crate::types::{BitMap256, CompressedDeck, DeckConfig};

static INIT_X1: Lazy<Vec<Uint256>> = Lazy::new(|| {
    vec![
        Uint256::from_str(
            "5299619240641551281634865583518297030282874472190772894086521144482721001553",
        )
        .unwrap(),
        Uint256::from_str(
            "10031262171927540148667355526369034398030886437092045105752248699557385197826",
        )
        .unwrap(),
        Uint256::from_str(
            "2763488322167937039616325905516046217694264098671987087929565332380420898366",
        )
        .unwrap(),
        Uint256::from_str(
            "12252886604826192316928789929706397349846234911198931249025449955069330867144",
        )
        .unwrap(),
        Uint256::from_str(
            "11480966271046430430613841218147196773252373073876138147006741179837832100836",
        )
        .unwrap(),
        Uint256::from_str(
            "10483991165196995731760716870725509190315033255344071753161464961897900552628",
        )
        .unwrap(),
        Uint256::from_str(
            "20092560661213339045022877747484245238324772779820628739268223482659246842641",
        )
        .unwrap(),
        Uint256::from_str(
            "7582035475627193640797276505418002166691739036475590846121162698650004832581",
        )
        .unwrap(),
        Uint256::from_str(
            "4705897243203718691035604313913899717760209962238015362153877735592901317263",
        )
        .unwrap(),
        Uint256::from_str(
            "153240920024090527149238595127650983736082984617707450012091413752625486998",
        )
        .unwrap(),
        Uint256::from_str(
            "21605515851820432880964235241069234202284600780825340516808373216881770219365",
        )
        .unwrap(),
        Uint256::from_str(
            "13745444942333935831105476262872495530232646590228527111681360848540626474828",
        )
        .unwrap(),
        Uint256::from_str(
            "2645068156583085050795409844793952496341966587935372213947442411891928926825",
        )
        .unwrap(),
        Uint256::from_str(
            "6271573312546148160329629673815240458676221818610765478794395550121752710497",
        )
        .unwrap(),
        Uint256::from_str(
            "5958787406588418500595239545974275039455545059833263445973445578199987122248",
        )
        .unwrap(),
        Uint256::from_str(
            "20535751008137662458650892643857854177364093782887716696778361156345824450120",
        )
        .unwrap(),
        Uint256::from_str(
            "13563836234767289570509776815239138700227815546336980653685219619269419222465",
        )
        .unwrap(),
        Uint256::from_str(
            "4275129684793209100908617629232873490659349646726316579174764020734442970715",
        )
        .unwrap(),
        Uint256::from_str(
            "3580683066894261344342868744595701371983032382764484483883828834921866692509",
        )
        .unwrap(),
        Uint256::from_str(
            "18524760469487540272086982072248352918977679699605098074565248706868593560314",
        )
        .unwrap(),
        Uint256::from_str(
            "2154427024935329939176171989152776024124432978019445096214692532430076957041",
        )
        .unwrap(),
        Uint256::from_str(
            "1816241298058861911502288220962217652587610581887494755882131860274208736174",
        )
        .unwrap(),
        Uint256::from_str(
            "3639172054127297921474498814936207970655189294143443965871382146718894049550",
        )
        .unwrap(),
        Uint256::from_str(
            "18153584759852955321993060909315686508515263790058719796143606868729795593935",
        )
        .unwrap(),
        Uint256::from_str(
            "5176949692172562547530994773011440485202239217591064534480919561343940681001",
        )
        .unwrap(),
        Uint256::from_str(
            "11782448596564923920273443067279224661023825032511758933679941945201390953176",
        )
        .unwrap(),
        Uint256::from_str(
            "15115414180166661582657433168409397583403678199440414913931998371087153331677",
        )
        .unwrap(),
        Uint256::from_str(
            "16103312053732777198770385592612569441925896554538398460782269366791789650450",
        )
        .unwrap(),
        Uint256::from_str(
            "15634573854256261552526691928934487981718036067957117047207941471691510256035",
        )
        .unwrap(),
        Uint256::from_str(
            "13522014300368527857124448028007017231620180728959917395934408529470498717410",
        )
        .unwrap(),
        Uint256::from_str(
            "8849597151384761754662432349647792181832839105149516511288109154560963346222",
        )
        .unwrap(),
        Uint256::from_str(
            "17637772869292411350162712206160621391799277598172371975548617963057997942415",
        )
        .unwrap(),
        Uint256::from_str(
            "17865442088336706777255824955874511043418354156735081989302076911109600783679",
        )
        .unwrap(),
        Uint256::from_str(
            "9625567289404330771610619170659567384620399410607101202415837683782273761636",
        )
        .unwrap(),
        Uint256::from_str(
            "19373814649267709158886884269995697909895888146244662021464982318704042596931",
        )
        .unwrap(),
        Uint256::from_str(
            "7390138716282455928406931122298680964008854655730225979945397780138931089133",
        )
        .unwrap(),
        Uint256::from_str(
            "15569307001644077118414951158570484655582938985123060674676216828593082531204",
        )
        .unwrap(),
        Uint256::from_str(
            "5574029269435346901610253460831153754705524733306961972891617297155450271275",
        )
        .unwrap(),
        Uint256::from_str(
            "19413618616187267723274700502268217266196958882113475472385469940329254284367",
        )
        .unwrap(),
        Uint256::from_str(
            "4150841881477820062321117353525461148695942145446006780376429869296310489891",
        )
        .unwrap(),
        Uint256::from_str(
            "13006218950937475527552755960714370451146844872354184015492231133933291271706",
        )
        .unwrap(),
        Uint256::from_str(
            "2756817265436308373152970980469407708639447434621224209076647801443201833641",
        )
        .unwrap(),
        Uint256::from_str(
            "20753332016692298037070725519498706856018536650957009186217190802393636394798",
        )
        .unwrap(),
        Uint256::from_str(
            "18677353525295848510782679969108302659301585542508993181681541803916576179951",
        )
        .unwrap(),
        Uint256::from_str(
            "14183023947711168902945925525637889799656706942453336661550553836881551350544",
        )
        .unwrap(),
        Uint256::from_str(
            "9918129980499720075312297335985446199040718987227835782934042132813716932162",
        )
        .unwrap(),
        Uint256::from_str(
            "13387158171306569181335774436711419178064369889548869994718755907103728849628",
        )
        .unwrap(),
        Uint256::from_str(
            "6746289764529063117757275978151137209280572017166985325039920625187571527186",
        )
        .unwrap(),
        Uint256::from_str(
            "17386594504742987867709199123940407114622143705013582123660965311449576087929",
        )
        .unwrap(),
        Uint256::from_str(
            "11393356614877405198783044711998043631351342484007264997044462092350229714918",
        )
        .unwrap(),
        Uint256::from_str(
            "16257260290674454725761605597495173678803471245971702030005143987297548407836",
        )
        .unwrap(),
        Uint256::from_str(
            "3673082978401597800140653084819666873666278094336864183112751111018951461681",
        )
        .unwrap(),
    ]
});

const SELECTOR0_BASE: u128 = 4_503_599_627_370_495;
const SELECTOR1_BASE: u128 = 3_075_935_501_959_818;

#[cw_serde]
pub struct Deck {
    pub config: DeckConfig,
    pub x0: Vec<Uint256>,
    pub x1: Vec<Uint256>,
    pub y0: Vec<Uint256>,
    pub y1: Vec<Uint256>,
    pub selector0: BitMap256,
    pub selector1: BitMap256,
    pub decrypt_record: Vec<BitMap256>,
    pub cards_to_deal: BitMap256,
    pub player_to_deal: u32,
}

pub fn initial_x1_values() -> &'static [Uint256] {
    &INIT_X1
}

pub fn card_index_from_x1(x1: &Uint256, config: DeckConfig) -> Option<u32> {
    let limit = config.num_cards() as usize;
    INIT_X1
        .iter()
        .take(limit)
        .position(|value| value == x1)
        .map(|idx| idx as u32)
}

impl Deck {
    pub fn new(config: DeckConfig) -> Self {
        let size = config.num_cards() as usize;
        let mut deck = Deck {
            config: config.clone(),
            x0: vec![Uint256::zero(); size],
            x1: INIT_X1[..size].to_vec(),
            y0: vec![Uint256::zero(); size],
            y1: vec![Uint256::zero(); size],
            selector0: BitMap256::zero(),
            selector1: BitMap256::zero(),
            decrypt_record: vec![BitMap256::zero(); size],
            cards_to_deal: BitMap256::zero(),
            player_to_deal: 0,
        };
        deck.selector0 = selector_for(config.num_cards(), SELECTOR0_BASE);
        deck.selector1 = selector_for(config.num_cards(), SELECTOR1_BASE);
        deck
    }

    pub fn size(&self) -> usize {
        self.config.num_cards() as usize
    }

    pub fn compressed(&self) -> CompressedDeck {
        CompressedDeck {
            config: self.config.clone(),
            x0: self.x0.clone(),
            x1: self.x1.clone(),
            selector0: self.selector0.clone(),
            selector1: self.selector1.clone(),
        }
    }

    pub fn set_from_compressed(&mut self, deck: CompressedDeck) -> StdResult<()> {
        if !deck.len_matches() {
            return Err(StdError::generic_err("compressed deck length mismatch"));
        }
        let size = deck.config.num_cards() as usize;
        self.config = deck.config.clone();
        self.x0 = deck.x0;
        self.x1 = deck.x1;
        self.selector0 = deck.selector0;
        self.selector1 = deck.selector1;
        self.y0.resize(size, Uint256::zero());
        self.y1.resize(size, Uint256::zero());
        self.decrypt_record.resize(size, BitMap256::default());
        Ok(())
    }
}

fn selector_for(deck_size: u32, base: u128) -> BitMap256 {
    let shift = 52u32 - deck_size;
    BitMap256::from_u128(base >> shift)
}

pub fn shuffle_public_input(
    enc: &CompressedDeck,
    old: &CompressedDeck,
    nonce: &Uint256,
    agg_pk_x: &Uint256,
    agg_pk_y: &Uint256,
) -> StdResult<Vec<Uint256>> {
    if enc.config != old.config {
        return Err(StdError::generic_err("deck config mismatch"));
    }
    if !enc.len_matches() || !old.len_matches() {
        return Err(StdError::generic_err("deck length mismatch"));
    }
    let deck_size = enc.config.num_cards() as usize;
    let mut input = Vec::with_capacity(7 + deck_size * 4);
    input.push(nonce.clone());
    input.push(agg_pk_x.clone());
    input.push(agg_pk_y.clone());
    for i in 0..deck_size {
        input.push(old.x0[i].clone());
    }
    for i in 0..deck_size {
        input.push(old.x1[i].clone());
    }
    for i in 0..deck_size {
        input.push(enc.x0[i].clone());
    }
    for i in 0..deck_size {
        input.push(enc.x1[i].clone());
    }
    input.push(old.selector0.data.clone());
    input.push(old.selector1.data.clone());
    input.push(enc.selector0.data.clone());
    input.push(enc.selector1.data.clone());
    Ok(input)
}
