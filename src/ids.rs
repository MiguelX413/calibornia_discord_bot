use serenity::all::{ChannelId, GuildId, RoleId};

pub(crate) const GUILD: GuildId = GuildId::new(980_962_249_550_213_170);

pub(crate) const INTROS: ChannelId = ChannelId::new(980_968_056_245_354_596);
pub(crate) const WELCOME_AND_RULES: ChannelId = ChannelId::new(980_962_249_550_213_172);
pub(crate) const GENERAL: ChannelId = ChannelId::new(980_962_249_550_213_176);
pub(crate) const SPAM: ChannelId = ChannelId::new(981_995_926_883_287_142);
pub(crate) const MODLOG: ChannelId = ChannelId::new(981_416_669_706_608_650);
pub(crate) const DAVEBOT: ChannelId = ChannelId::new(1_089_751_694_352_584_725);

pub(crate) const ADMIN: RoleId = RoleId::new(980_964_927_164_518_470);
pub(crate) const MEMBER: RoleId = RoleId::new(982_177_726_691_700_736);
pub(crate) const MOD: RoleId = RoleId::new(1_027_089_314_405_957_685);
const COLOR_DIVIDER: RoleId = RoleId::new(1_027_311_103_014_862_888);
const LOCATION_DIVIDER: RoleId = RoleId::new(1_027_310_335_314_628_708);
const PING_DIVIDER: RoleId = RoleId::new(1_027_095_201_262_616_607);
const PRONOUN_DIVIDER: RoleId = RoleId::new(1_027_094_772_848_005_160);
const CLASSPECT_DIVIDER: RoleId = RoleId::new(1_027_309_033_373_310_987);
const MISC_DIVIDER: RoleId = RoleId::new(1_027_309_906_807_750_676);
pub(crate) const UNVERIFIED: RoleId = RoleId::new(1_098_091_859_743_612_948);

pub(crate) const DEFAULT_JOIN_ROLES: &[RoleId] = &[
    COLOR_DIVIDER,
    LOCATION_DIVIDER,
    PING_DIVIDER,
    PRONOUN_DIVIDER,
    CLASSPECT_DIVIDER,
    MISC_DIVIDER,
    UNVERIFIED,
];
