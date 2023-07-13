use memoize::memoize;

use super::*;

#[memoize]
pub fn precomputed_magic_value_for_index_and_piece(
    piece_index: usize,
    piece_for_magic: WalkingPieces,
) -> MagicValue {
    MagicValue {
        magic: DEFAULT_MAGICS[piece_for_magic as usize][piece_index as usize],
        bits_required: DEFAULT_BITS_REQUIRED[piece_for_magic as usize][piece_index as usize],
    }
}

const DEFAULT_MAGICS: [[u64; 64]; 2] = [
    [
        36031547412840464,
        1459166897755922432,
        36063981530513536,
        36033745089857536,
        1297041125359486464,
        4683770155363270670,
        144172366984380928,
        36029347078881536,
        145381843230408705,
        618400672093847560,
        153403999749605440,
        2315131722102214656,
        18295942276448512,
        9570218068279332,
        9872171877651188224,
        1264526354347475072,
        4620799320831049728,
        166633461095014402,
        289638850955837506,
        1268286931093504,
        9223654611745771524,
        36311371541118984,
        4611831154096558090,
        5764609722130956420,
        2322175000340629,
        9223407222300626944,
        18302552162435266,
        4107573133382254724,
        10376312237454526464,
        578994065749311498,
        9223373153547323394,
        738591172114579604,
        1194052310733422720,
        149671062290435,
        5228821141958692868,
        2269426929910272,
        162270616156653568,
        140754676613632,
        360781614944816,
        9223971274483564676,
        2341889535861866496,
        162129758652473348,
        301749988308819969,
        4656901303832739848,
        4692756309412478993,
        613615520635420936,
        2253002673225730,
        72339071167037442,
        108086976246228608,
        4765019512528440384,
        9227946417611669760,
        4574312136745472,
        140754668748928,
        562984581923328,
        1369114112307512320,
        72479809740996736,
        578996296086454305,
        1441715936670259234,
        283811460988937,
        432491391253745665,
        54607384647569410,
        2306405993795554050,
        6142912125389021444,
        22800053070726210,
    ],
    [
        144713331326861440,
        2328418220677013504,
        11819717018646478889,
        152588033286259202,
        38360037120278560,
        1306326879029952520,
        1158569163531749440,
        1225122039612834816,
        9232449641495167105,
        576500369117495816,
        45053592788279296,
        9295438452759724040,
        18159723559789088,
        2306125077713780993,
        4508581923786752,
        4900092592426360842,
        797278696435679520,
        581038199666114982,
        74311661603786800,
        108227300377772040,
        111464099654666240,
        13546129552574473,
        324553859484090900,
        288353798546589760,
        147529721537105920,
        360639889934368,
        583229348065314832,
        1161937500072067209,
        1226387573041020928,
        1441721436393177600,
        290484375282287620,
        73183768964826129,
        72656140681347588,
        2317111087979562016,
        2598014311396017408,
        1729529593618497664,
        1477270839341744256,
        148636998903136768,
        37155255305275408,
        24858035918798976,
        2921420115951624,
        9185324970365088,
        1155793429582583808,
        144115325800024081,
        9016008237646848,
        9224674419148849920,
        1233991331739598976,
        364811363190720050,
        2900600803519963136,
        2306125618096504840,
        866098645207360514,
        4612248972751798272,
        68753162432,
        576531310164676640,
        9516178623955207681,
        9372021623835599489,
        577877069577459712,
        741074276976696,
        567906379301952,
        155374604297306641,
        109566357867217928,
        148619372091484676,
        9224533138582143040,
        4504712158150912,
    ],
];

const DEFAULT_BITS_REQUIRED: [[usize; 64]; 2] = [
    [
        12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10,
        11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10,
        10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
    ],
    [
        6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7,
        5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5,
        5, 5, 5, 6,
    ],
];
