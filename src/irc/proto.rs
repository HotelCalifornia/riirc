use bytes::BytesMut;
use num_enum::TryFromPrimitive;
use std::{collections::HashMap, convert::TryFrom, time};

pub enum ModeType {}

pub enum UserMode {
    A(ModeType, Option<String>),
    B(ModeType, String),
    C(ModeType, Option<String>),
    D(ModeType),
}

pub enum ChannelMode {
    A(ModeType, Option<String>),
    B(ModeType, String),
    C(ModeType, Option<String>),
    D(ModeType),
}

pub enum Mode {
    /// user mode: true -> +, false -> -
    User(bool, UserMode),
    /// channel mode: true -> +, false -> -
    Channel(bool, ChannelMode),
}

pub enum Command {
    // connection commands

    /// CAP subcommand [:capabilities] - capabilities negotiation
    Cap(String, Option<Vec<String>>),
    /// AUTHENTICATE - SASL authentication
    Authenticate(()),
    /// PASS password - provide password to server
    Pass(String),
    /// NICK nickname - set nickname
    Nick(String),
    /// USER username 0 * [:real name] - specify username and realname (NOTE: the second and third parameters are
    ///     hardcoded as their meanings differs per specification version)
    User(String, Option<String>),
    /// OPER name password - obtain operator privileges
    Oper(String, String),
    /// QUIT [reason] - disconnect from the server, optionally with a reason
    Quit(Option<String>),

    // channel commands

    /// JOIN channel{,channel}* [key{,key}*] - join one or more channels, optionally using the given keys
    Join(Vec<String>, Vec<String>),
    /// PART channel{,channel}* [reason] - leave one or more channels, optionally with a reason
    Part(Vec<String>, String),
    /// TOPIC channel [topic] - get or set a channel's topic (set if topic parameter is specified, else get)
    Topic(String, Option<String>),
    /// NAMES channel - get the nicknames joined to a channel (NOTE: the specification technically allows asking for
    ///     zero or more channels, but the response for this command sent with zero channels specified is widely
    ///     variant, and most servers ignore requests for more than one channel nowadays)
    Names(String),
    /// LIST [channel{,channel}*] [elistcond{,elistcond}*] - get a list of channels and some information about each one
    ///     (TODO: handle elistcond?)
    List(Vec<String>),

    // server queries and commands

    /// MOTD [target] - display MOTD of the specified server (or the currently connected server, if unspecified)
    Motd(Option<String>),
    /// VERSION [target] - request version and ISupport parameters of the specified server (or the currently connected 
    ///     server, if unspecified)
    Version(Option<String>),
    /// ADMIN [target] - find the name of the administrator of the specified server (or the currently connected server,
    ///     if unspecified)
    Admin(Option<String>),
    /// CONNECT target [port [remote]] - force the server to attempt a connection to a target server, optionally giving
    ///     a port number and (if specified) a server that will be forced to attempt the connection (i.e. instead of the
    ///     current one)
    Connect(String, Option<(String, Option<String>)>),
    /// TIME [server] - query local time from specified server (or the server that handles the query, if unspecified)
    Time(Option<String>),
    /// STATS query [server] - query an optionally specified server for statistics (NOTE: see
    ///     https://modern.ircdocs.horse/#stats-message )
    Stats(String, Option<String>),
    /// INFO [target] - get information about the optionally speficied server (or the server that handles the request,
    ///     if unspecified)
    Info(Option<String>),
    /// MODE target [modestring [modeargs...]] - set or remove modes on/from a given target
    Mode(String, Mode),
    /// PRIVMSG target{,target}* :message text - send a message to a target or targets
    PrivMsg(Vec<String>, String),
    /// NOTICE target{,target}* :notice text - send a notice to a target or targets (NOTE: NOTICEs are similar to
    ///     PRIVMSGs, with the difference that automatic replies must never be sent in response to a NOTICE)
    Notice(Vec<String>, String),

    // optional messages may not be implemented by servers

    /// USERHOST nickname{ nickname}* - get information about up to five nicknames
    UserHost(Vec<String>),
    
    // miscellaneous messages

    /// KILL nickname comment - close the connection between a given client and the server to which they are connected
    ///     (NOTE: only available to server operators)
    Kill(String, String),
}

pub enum Numeric {
    Welcome(String, String),
    YourHost(String, String),
    Created(String, String),
    /// RPL_MYINFO (004): client servername version usermodes channelmodes [paramchannelmodes]
    MyInfo(String, String, String, String, String, Option<String>),
    /// RPL_ISUPPORT (005): client token{ token}{0,12} :message
    ISupport(String, Vec<String>, String),
    /// RPL_BOUNCE (010): client hostname port :info
    Bounce(String, String, String, String),
    /// RPL_UMODEIS (221): client usermodes
    UModeIs(String, Vec<UserMode>),
    /// RPL_STATSDLINE (250): client :info - per RFC 2812, used by EsperNet at least to inform client of highest
    ///     connection count and total(?) number of connections made
    StatsDLine(String, String),
    /// RPL_LUSERCLIENT (251): client :info
    LUserClient(String, String),
    /// RPL_LUSEROP (252): client numops :message
    LUserOp(String, u32, String),
    /// RPL_LUSERUNKNOWN (253): client numconns :message
    LUserUnknown(String, u32, String),
    /// RPL_LUSERCHANNELS (254): client numchans :message
    LUserChannels(String, u32, String),
    /// RPL_LUSERME (255): client :info
    LUserMe(String, String),
    /// RPL_LADMINME (256): client [server] :admin info
    LAdminMe(String, Option<String>, String),
    AdminLoc1(String, String),
    AdminLoc2(String, String),
    AdminEmail(String, String),
    /// RPL_TRYAGAIN (263): client command :message
    TryAgain(String, String, String),
    /// RPL_LOCALUSERS (265): client [numclients maxclients] :message
    LocalUsers(String, Option<(u32, u32)>, String),
    /// RPL_GLOBALUSERS (266): client [numclients maxclients] :message
    GlobalUsers(String, Option<(u32, u32)>, String),
    /// RPL_WHOISCERTFP (276): client nickname :message
    WhoIsCertFP(String, String, String),
    None(()),
    /// RPL_AWAY (301): client nickname :message
    Away(String, String, String),
    /// RPL_USERHOST (302): client :[reply{ reply}*]
    UserHost(String, String),
    /// RPL_ISON (303): client :[nickname{ nickname}*]
    IsOn(String, String),
    /// RPL_UNAWAY (305): client :message
    UnAway(String, String),
    /// RPL_NOWAWAY (306): client :message
    NowAway(String, String),
    /// RPL_WHOISUSER (311): client nickname username host * :real name
    WhoIsUser(String, String, String, String, String),
    /// RPL_WHOISSERVER (312): client nickname server :server info
    WhoIsServer(String, String, String, String),
    /// RPL_WHOISOPERATOR (313): client nickname :info
    WhoIsOperator(String, String, String),
    /// RPL_WHOWASUSER (314): client nickname username host * :real name
    WhoWasUser(String, String, String, String, String),
    /// RPL_WHOISIDLE (317): client nickname seconds [signon] :message
    WhoIsIdle(String, String, time::Duration, Option<time::Instant>, String),
    /// RPL_ENDOFWHOIS (318): client nickname :message
    EndOfWhoIs(String, String, String),
    /// RPL_WHOISCHANNELS (319): client nickname :[prefix]channel{ [prefix]channel}*
    WhoIsChannels(String, String, String),
    // ListStart()
}

#[repr(u16)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum InfoReply {
    Welcome         = 001,
    YourHost,
    Created,
    MyInfo,
    ISupport,
    Bounce          = 010,
    UModeIs         = 221,
    /// Per RFC 2812, used by EsperNet at least to inform client of highest connection count and total(?) number of connections received
    StatsDLine      = 250,
    LUserClient,
    LUserOp,
    LUserUnknown,
    LUserChannels,
    LUserMe,
    LAdminMe,
    AdminLoc1,
    AdminLoc2,
    AdminEmail,
    TryAgain        = 263,
    LocalUsers      = 265,
    GlobalUsers,
    WhoIsCertFP,
}

#[repr(u16)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum CommandReply {
    None            = 300,
    Away,
    UserHost,
    IsOn,
    UnAway,
    NowAway,
    WhoIsUser       = 311,
    WhoIsServer,
    WhoIsOperator,
    WhoWasUser,
    WhoIsIdle       = 317,
    EndOfWhoIs,
    WhoIsChannels,
    ListStart       = 321,
    List,
    ListEnd,
    ChannelModeIs,
    /// Attested to AUSTnet and Bahamut IRCd implementations
    ChannelUrl      = 328,
    CreationTime,
    NoTopic         = 331,
    Topic,
    TopicWhoTime,
    Inviting        = 341,
    InviteList      = 346,
    EndOfInviteList,
    ExceptList,
    EndOfExceptList,
    Version         = 351,
    NameReply       = 353,
    EndOfNames      = 366,
    BanList,
    EndOfBanList,
    EndOfWhoWas,
    MOTD            = 372,
    MOTDStart       = 375,
    EndOfMOTD,
    YoureOperator   = 381,
    Rehashing,
}

#[repr(u16)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum ErrorReply {
    Unknown             = 400,
    NoSuchNick,
    NosuchServer,
    NoSuchChannel,
    CannotSendToChannel,
    TooManyChannels,
    UnknownCommand      = 421,
    NoMOTD,
    ErroneousNickname   = 432,
    NickInUse,
    UserNotInChannel    = 441,
    NotOnChannel,
    UserOnchannel,
    NotRegistered       = 451,
    NeedMoreParameters  = 461,
    AlreadRegistered,
    PasswordMismatch    = 464,
    YoureBannedCreep,
    ChannelIsFull       = 471,
    UnknownMode,
    InviteOnlyChannel,
    BannedFromChannel,
    BadChannelKey,
    /// IRC user is not an operator and thus does not have permission to perform requested action
    NoPrivileges        = 481,
    ChanOpPrivsNeeded,
    CantKillServer,
    NoOperHost          = 491,
    UModeUnknownFlag    = 501,
    UsersDontMatch,
    /// IRCv3 tls extension: client may start TLS handshake
    StartTLS            = 691,
    /// IRC operator does not have specific permission to perform requested action
    NoPrivs             = 723,
    /// IRCv3 sasl-3.1 extension: SASL authentication failed because account is locked out
    NickLocked          = 902,
    /// IRCv3 sasl-3.1 extension: SASL authentication failed because of invalid credentials or other unspecified reason
    SASLFail            = 904,
    /// IRCv3 sasl-3.1 extension: SASL authentication failed because the AUTHENTICATE command sent by the client was too long (param > 400 B)
    SASLTooLong,
    /// IRCv3 sasl-3.1 extension: SASL authentication failed because the client sent an AUTHENTICATE command with the parameter `0x2A` (`'*'`)
    SASLAborted,
    /// IRCv3 sasl-3.1 and sasl-3.2 extension: SASL authentication failed because the client is already authenticated and reauthentication is disabled
    SASLAlready,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reply {
    Info(InfoReply),
    Command(CommandReply),
    Error(ErrorReply),
}

impl From<u16> for Reply {
    fn from(n: u16) -> Self {
        if let Ok(ir) = InfoReply::try_from(n) {
            Reply::Info(ir)
        } else if let Ok(cr) = CommandReply::try_from(n) {
            Reply::Command(cr)
        } else if let Ok(er) = ErrorReply::try_from(n) {
            Reply::Error(er)
        } else {
            panic!("unknown reply {}", n)
        }
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Cmd(String),
    Response(Reply),
}

impl From<BytesMut> for Command {
    fn from(src: BytesMut) -> Self {
        match src[0] {
            b'0'..=b'9' => Command::Response(Reply::from(String::from_utf8(src.to_vec()).unwrap().parse::<u16>().unwrap())),
            _ => Command::Cmd(String::from_utf8(src.to_vec()).unwrap()),
        }
    }
}

impl From<String> for Command {
    fn from(src: String) -> Self {
        match src.parse::<u16>() {
            Ok(n) => Command::Response(Reply::from(n)),
            Err(_) => Command::Cmd(src),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub tags: HashMap<String, String>,
    pub prefix: Option<String>,
    pub command: Command,
    pub params: Vec<String>,
}

impl From<BytesMut> for Message {
    fn from(src: BytesMut) -> Self {
        let src_str = String::from_utf8(src.to_vec()).unwrap();
        // println!(">> consctructing Message from {}", src_str);

        // extract tags: (@(\S+(=\S+)?)?(;\S+(=\S+)?)*)?
        let mut tags = HashMap::new();
        if src_str.starts_with("@") {
            let next = src_str.find(" ").expect("malformed message");
            let raw_tags: Vec<&str> = src_str[1..next].split(";").collect();
            for tag in raw_tags {
                let _t: Vec<&str> = tag.split("=").collect();
                if _t.len() > 1 {
                    tags.insert(String::from(_t[0]), String::from(_t[1]));
                } else {
                    tags.insert(String::from(_t[0]), String::from("true"));
                }
            }
        }
        // println!(">> tags: {:#?}", tags);

        // extract prefix: (:\S+)?
        let mut src_str = String::from(src_str.trim_start());
        let prefix = if src_str.starts_with(":") {
            let next = src_str.find(" ").expect("malformed message");
            let r = Some(String::from(&src_str[1..next]));
            src_str = String::from(src_str.trim_start_matches(&src_str[0..next]));
            r
        } else {
            None
        };
        // println!(">> prefix: {:?}", prefix);

        // extract command: \S+
        let src_str = String::from(src_str.trim_start());
        let next = src_str.find(" ").map_or_else(|| src_str.len(), |i| i);
        let command = Command::from(String::from(&src_str[0..next]));

        // println!(">> command: {:?}", command);
        
        // extract params: (\S+\s+){0,14}(:.+)?
        let src_str = src_str.trim_start_matches(&src_str[0..next]);
        // println!(">> still to parse: {}", src_str);

        let src_str = String::from(src_str.trim_start());

        let (s, t) = if let Some(i) = src_str.find(":") {
            let (s, t) = src_str.split_at(i);
            (String::from(s), String::from(t))
        } else {
            (src_str, String::from(""))
        };
        let src_str = s;
        let trailing = String::from(t.chars().next().map(|c| &t[c.len_utf8()..]).unwrap_or(""));

        let mut params = src_str.split(" ").map(|s| String::from(s)).filter(|s| !s.is_empty()).collect::<Vec<String>>();
        params.push(trailing);

        // println!(">> params: {:?}", params);

        Message {
            tags,
            prefix,
            command,
            params,
        }
    }
}

impl From<Message> for BytesMut {
    fn from(msg: Message) -> Self {
        // encode tags
        let tags = if !msg.tags.is_empty() {
            format!("@{} ", msg.tags.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join(";"))
        } else {
            String::from("")
        };

        // encode prefix
        let prefix = if let Some(p) = msg.prefix {
            format!(":{} ", p)
        } else {
            String::from("")
        };

        // encode command
        let command = match msg.command {
            Command::Response(n) => format!("{:?}", n),
            Command::Cmd(s) => format!("{}", s),
        };

        // encode params
        let params = if let Some((last, elements)) = msg.params.split_last() {
            format!("{} :{}", elements.join(" "), last)
        } else {
            String::from("")
        };

        BytesMut::from(format!("{}{}{} {}\r\n", tags, prefix, command, params).as_bytes())
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub nick: String,
    pub name: Option<String>,
    pub real_name: Option<String>,
}

impl User {
    pub fn new(nick: String, name: Option<String>, real_name: Option<String>) -> Self {
        User{
            nick, name, real_name
        }
    }
}