//! The client's own texts, in every language it speaks. English is the default; a text missing in another
//! language cannot happen, as every key carries all of them.
//!
//! The wording of what the original client also says follows its `msgstringtable.txt`.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Ru];

    /// The code kept in the browser's storage and shown on the language switch.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|lang| code.eq_ignore_ascii_case(lang.code()))
    }
}

macro_rules! texts {
    ($($key:ident: $en:literal, $ru:literal;)*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Key { $($key),* }

        impl Key {
            pub const ALL: &'static [Key] = &[$(Key::$key),*];

            pub fn text(self, lang: Lang) -> &'static str {
                match (self, lang) {
                    $((Key::$key, Lang::En) => $en, (Key::$key, Lang::Ru) => $ru,)*
                }
            }
        }
    };
}

texts! {
    // The login window.
    Service: "TenThousand", "TenThousand";
    Password: "PASSWORD", "ПАРОЛЬ";
    SaveId: "Save ID", "Запомнить ID";
    Register: "Register", "Регистрация";
    Restore: "Restore", "Забыл";
    EnterAtLeast: "Please enter at least {} characters.", "Введите не меньше {} символов.";
    PleaseWait: "Please wait...", "Пожалуйста, подождите...";

    // Why a server said no.
    UnregisteredId: "Unregistered ID. Please make sure you have a registered account and you have correctly typed in the user ID.",
        "Такой ID не зарегистрирован. Проверьте, что аккаунт создан и ID введён правильно.";
    WrongPassword: "Incorrect User ID or Password. Please try again.", "Неверный ID или пароль. Попробуйте ещё раз.";
    IdExpired: "This ID is expired.", "Срок действия этого ID истёк.";
    Rejected: "Rejected from server.", "Сервер отклонил подключение.";
    Blocked: "You have been blocked by the GM Team.", "Аккаунт заблокирован администрацией.";
    BlockedUntil: "You are prohibited to log in until {}.", "Вход запрещён до {}.";
    Overpopulated: "Server is jammed due to overpopulation. Please try again after few minutes.", "Сервер переполнен. Попробуйте через несколько минут.";
    RejectedCode: "Rejected from server ({}).", "Сервер отклонил подключение ({}).";
    ServerClosed: "Server closed.", "Сервер закрыт.";
    AlreadyLoggedIn: "Someone has already logged in with this ID.", "С этим ID уже вошли в игру.";
    StillRecognized: "Server still recognizes your last login. Please try again in a moment.", "Сервер ещё помнит ваш прошлый вход. Попробуйте чуть позже.";
    Disconnected: "Disconnected from server.", "Соединение с сервером разорвано.";
    DisconnectedCode: "Disconnected from server ({}).", "Соединение с сервером разорвано ({}).";
    ConnectFailed: "Failed to connect to server.", "Не удалось подключиться к серверу.";
    NoCharServer: "No character server is available.", "Нет доступного сервера персонажей.";
    SelectServer: "Select Server", "Выбор сервера";
    TooManyLogins: "Too many failed login attempts. Please try again in {} min.", "Слишком много неудачных попыток входа. Попробуйте через {} мин.";

    // Accounts.
    RegisterTitle: "Account Registration", "Регистрация аккаунта";
    RestoreTitle: "Password Recovery", "Восстановление пароля";
    ResetTitle: "New Password", "Новый пароль";
    Id: "ID", "ID";
    PasswordLabel: "Password", "Пароль";
    RepeatPassword: "Repeat", "Ещё раз";
    Email: "E-mail", "E-mail";
    Sex: "Sex", "Пол";
    Male: "Male", "Мужской";
    Female: "Female", "Женский";
    Ok: "OK", "OK";
    Cancel: "Cancel", "Отмена";
    BadId: "An ID is 4 to 23 letters, digits and underscores.", "ID — от 4 до 23 латинских букв, цифр и знаков подчёркивания.";
    BadPassword: "A password is 8 to 23 letters, digits and symbols, without spaces.", "Пароль — от 8 до 23 латинских букв, цифр и символов, без пробелов.";
    PasswordsDiffer: "The two passwords are not the same.", "Пароли не совпадают.";
    BadEmail: "Please enter a valid e-mail address of at most 39 characters.", "Введите правильный e-mail не длиннее 39 символов.";
    AccountCreated: "Your account has been created. You can log in now.", "Аккаунт создан. Теперь можно войти в игру.";
    IdTaken: "This ID is already in use. Please choose another one.", "Этот ID уже занят. Выберите другой.";
    RestoreHint: "Enter your ID and the e-mail address of your account. A link to set a new password will be sent to it.",
        "Введите ID и e-mail аккаунта. На него придёт ссылка для установки нового пароля.";
    RestoreSent: "If the ID and e-mail address belong to an account, a message with a link to set a new password is on its way.",
        "Если ID и e-mail принадлежат аккаунту, письмо со ссылкой для установки нового пароля уже отправлено.";
    PasswordChanged: "Your password has been changed. You can log in now.", "Пароль изменён. Теперь можно войти в игру.";
    LinkInvalid: "This link is not valid any more. Please ask for a new one.", "Эта ссылка больше не действует. Запросите новую.";
    TooManyAttempts: "Too many attempts. Please try again in {} min.", "Слишком много попыток. Попробуйте через {} мин.";
    CheckForm: "Please check what you have entered.", "Проверьте введённые данные.";
    ServiceUnavailable: "The account service is not available right now. Please try again later.", "Сервис аккаунтов сейчас недоступен. Попробуйте позже.";

    // Character select.
    CharacterList: "Character List", "Список персонажей";
    GameStart: "Game Start", "Начать игру";
    DeleteCharacter: "Delete Character", "Удалить персонажа";
    Notice: "Notice", "Объявление";

    CancelDeletion: "Cancel Deletion", "Отменить удаление";
    Delete: "Delete", "Удалить";
    AskDelete: "Do you want to delete the character {}? It can be deleted for good after a waiting time, and until then the deletion can be cancelled.",
        "Удалить персонажа {}? Окончательно удалить его можно будет после срока ожидания, а до тех пор удаление можно отменить.";
    AskDeleteNow: "Delete the character {} for good? This cannot be undone.", "Удалить персонажа {} окончательно? Это нельзя отменить.";
    DeleteGuild: "This character cannot be deleted while it is in a guild.", "Нельзя удалить персонажа, пока он состоит в гильдии.";
    DeleteParty: "This character cannot be deleted while it is in a party.", "Нельзя удалить персонажа, пока он состоит в группе.";
    DeleteFailed: "The character could not be deleted.", "Не удалось удалить персонажа.";
    DeleteNotYet: "It is not yet time to delete this character.", "Срок ожидания удаления ещё не истёк.";
    AskBirthdate: "Enter your date of birth (YYMMDD) to delete the character.", "Введите дату рождения (ГГММДД), чтобы удалить персонажа.";
    WrongBirthdate: "The date of birth is not correct.", "Дата рождения указана неверно.";
    Deleted: "The character has been deleted.", "Персонаж удалён.";
    DeletionReady: "Can be deleted", "Можно удалить";
    TooNarrow: "The game needs a window at least {} pixels wide. Please make the window wider.",
        "Игре нужно окно шириной не меньше {} пикселей. Сделайте окно шире.";
    Sound: "Sound", "Звук";
    SoundHint: "Click anywhere to start the music", "Нажмите в любом месте, чтобы включить музыку";

    // Character creation.
    CharacterCreation: "Character Creation", "Создание персонажа";
    Human: "Human", "Человек";
    Doram: "Doram", "Дорам";
    HumanAbout: "Dominant race of Rune Midgard. Unlimited potential. Excellent adaptability to solve problems.",
        "Главная раса Рун-Мидгарда. Безграничный потенциал и умение приспособиться к любой задаче.";
    DoramAbout: "Race of Pasta continent with natural born curiosity and sparkie character.",
        "Раса с континента Паста: прирождённое любопытство и искромётный характер.";
    HairStyle: "Hair Style", "Причёска";
    HairColor: "Hair Color", "Цвет волос";
    Create: "Create", "Создать";
    Return: "Return", "Назад";
    BadName: "A character name is 4 to 23 letters and digits.", "Имя персонажа — от 4 до 23 латинских букв и цифр.";
    NameExists: "Character name already exists.", "Персонаж с таким именем уже есть.";
    Underaged: "You are underaged and cannot join this server.", "Вы слишком молоды для этого сервера.";
    SymbolsForbidden: "Symbols in character names are forbidden.", "Символы в именах персонажей запрещены.";
    SlotNotAllowed: "You are not eligible to open the character slot.", "Этот слот персонажа вам недоступен.";
    CreationDenied: "Character creation is denied.", "Создание персонажа отклонено.";

    // In the game, until there is a world to show.
    LogOut: "Log out", "Выйти";
}

/// Puts `values` into the text's `{}` places, in order.
pub fn fill(text: &str, values: &[&str]) -> String {
    let mut filled = String::with_capacity(text.len() + 8);
    let mut values = values.iter();
    let mut rest = text;
    while let Some(at) = rest.find("{}") {
        filled.push_str(&rest[..at]);
        filled.push_str(values.next().copied().unwrap_or_default());
        rest = &rest[at + 2..];
    }
    filled.push_str(rest);
    filled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_there_in_every_language() {
        for &key in Key::ALL {
            let places = key.text(Lang::En).matches("{}").count();
            for lang in Lang::ALL {
                let text = key.text(lang);
                assert!(!text.trim().is_empty(), "{key:?} in {lang:?}");
                assert_eq!(text.matches("{}").count(), places, "{key:?} in {lang:?} takes other values than in English");
            }
        }
    }

    #[test]
    fn russian_is_russian() {
        let cyrillic = Key::ALL.iter().filter(|key| key.text(Lang::Ru).chars().any(|character| ('а'..='я').contains(&character))).count();
        // All but the few words that stay as they are (names, "ID", "OK", "E-mail").
        assert!(cyrillic >= Key::ALL.len() - 6, "{cyrillic} of {}", Key::ALL.len());
    }

    #[test]
    fn languages_are_found_by_code() {
        assert_eq!(Lang::from_code("ru"), Some(Lang::Ru));
        assert_eq!(Lang::from_code("EN"), Some(Lang::En));
        assert_eq!(Lang::from_code("de"), None);
        assert_eq!(Lang::default(), Lang::En);
    }

    #[test]
    fn fills_values_in_order() {
        assert_eq!(fill("from {} to {}.", &["4", "23"]), "from 4 to 23.");
        assert_eq!(fill("no places", &["x"]), "no places");
        assert_eq!(fill("{} and {}", &["one"]), "one and ");
    }
}

thread_local! {
    /// The language the game's windows speak now.
    static NOW: std::cell::Cell<Lang> = const { std::cell::Cell::new(Lang::En) };
}

/// Sets the language the game's windows speak.
pub fn speak(lang: Lang) {
    NOW.set(lang);
}

/// The language the game's windows speak now.
pub fn spoken() -> Lang {
    NOW.get()
}

/// A word or phrase of the game's windows in the language spoken now. The windows are written in English, as
/// the client's own tables are; what has no translation stays as it is.
pub fn tr(english: &str) -> &str {
    if NOW.get() == Lang::En {
        return english;
    }
    PHRASES.iter().find(|(known, _)| *known == english).map_or(english, |(_, russian)| russian)
}

/// The phrases of the game's windows, in English and in Russian.
const PHRASES: &[(&str, &str)] = &[
    ("online", "в сети"),
    ("Players online", "Игроков в сети"),
    ("Language", "Язык"),
    ("Game Settings", "Настройки игры"),
    ("Basic", "Основное"),
    ("Effects", "Эффекты"),
    ("Controls", "Управление"),
    ("Graphics", "Графика"),
    ("Etc", "Прочее"),
    ("Audio Settings", "Звук"),
    ("BGM", "Музыка"),
    ("SFX", "Звуки"),
    ("Off", "Выкл."),
    ("The browser sets the graphics; there is nothing to choose here.", "Графику задаёт браузер: выбирать здесь нечего."),
    ("Check Pet Status", "Сведения о питомце"),
    ("Shortcut Settings", "Настройка клавиш"),
    ("Hotkey Bar", "Панель"),
    ("Hotkey Bar 2", "Панель 2"),
    ("Interface", "Интерфейс"),
    ("Macros", "Макросы"),
    ("Not Assigned", "Не назначено"),
    ("Press a key to assign. Pressing 'ESC' will remove the assigned key.", "Нажмите клавишу, чтобы назначить её. ESC снимает назначение."),
    ("This keyboard does not have that key.", "На этой клавиатуре нет такой клавиши."),
    ("Basic Information", "Основная информация"),
    ("Game Options", "Настройки игры"),
    ("Equipment", "Экипировка"),
    ("Storage", "Склад"),
    ("Inventory", "Инвентарь"),
    ("Skill Tree", "Дерево умений"),
    ("Status", "Характеристики"),
    ("Do you really want to go back to your savepoint?", "Вы действительно хотите вернуться к точке сохранения?"),
    ("Amount", "Кол-во"),
    ("Available Items to sell", "Предметы на продажу"),
    ("Selling Items", "Продаваемые предметы"),
    ("Available Items to buy", "Предметы в продаже"),
    ("Buying Items", "Покупаемые предметы"),
    ("Unknown Skill", "Неизвестное умение"),
    ("Item", "Предмет"),
    ("Skill", "Умение"),
    ("Cash", "Кэш"),
    ("Armor", "Броня"),
    ("Wpn", "Оруж"),
    ("Cstm", "Кост"),
    ("Ammo", "Снар"),
    ("Card", "Карты"),
    ("Misc", "Проч"),
    ("Use", "Исп"),
    ("Eqp", "Экип"),
    ("Etc", "Проч"),
    ("1st", "1-я"),
    ("2nd", "2-я"),
    ("3rd", "3-я"),
    ("4th", "4-я"),
    ("Public Log", "Общий"),
    ("Battle Log", "Бой"),
    ("Hide the log", "Скрыть журнал"),
    ("Add a tab", "Добавить вкладку"),
    ("Detach", "Открепить"),
    ("Options", "Настройки"),
    ("Base Lv.", "Б. ур."),
    ("Job Lv.", "П. ур."),
    ("Lv.", "Ур."),
    ("Exp.", "Опыт"),
    ("Weight", "Вес"),
    ("Zeny", "Зени"),
    ("ea", "шт."),
    ("Skill Point", "Очки умений"),
    ("obtained.", "получено."),
    ("Raise", "Повысить"),
    ("Log out", "Выход"),
    ("You cannot leave the game right now.", "Сейчас нельзя выйти из игры."),
    ("There is no such character name or the user is offline.", "Такого персонажа нет, или игрок не в сети."),
    ("Skill has failed.", "Умение не удалось."),
    ("Pet Egg List", "Яйца питомцев"),
    ("Identify Item", "Опознание предмета"),
    ("Refinable", "Можно точить"),
    ("Not refinable", "Нельзя точить"),
    ("Enchantable", "Можно зачаровать"),
    ("Not enchantable", "Нельзя зачаровать"),
    ("Grade enchantable", "Можно повысить ранг"),
    ("Not grade enchantable", "Нельзя повысить ранг"),
    ("Pet Info", "Питомец"),
    ("Name", "Имя"),
    ("Level", "Ур."),
    ("Hunger", "Голод"),
    ("Intimacy", "Связь"),
    ("Accessory", "Аксессуар"),
    ("Equipped", "Надет"),
    ("Unequipped", "Не надет"),
    ("Very Hungry", "Очень голоден"),
    ("Hungry", "Голоден"),
    ("Neutral", "Обычно"),
    ("Satisfied", "Сыт"),
    ("Stuffed", "Объелся"),
    ("Awkward", "Чужой"),
    ("Shy", "Робкий"),
    ("Cordial", "Дружелюбный"),
    ("Loyal", "Преданный"),
    ("Unknown", "Неизвестно"),
    ("Feed Pet", "Покормить"),
    ("Performance", "Трюк"),
    ("Unequip Accessory", "Снять аксессуар"),
    ("Return to Egg", "Вернуть в яйцо"),
    ("Are you sure that you want to feed your pet?", "Покормить питомца?"),
    ("Pet name will be changed to {}, do you want to continue?", "Питомца будут звать {}. Продолжить?"),
    ("Your pet name must be 23 characters or less.", "Имя питомца — не длиннее 23 символов."),
    ("Your pet has been fed. {} ({}) remaining", "Питомец накормлен. {}: осталось {}"),
    ("Pet food [{}] is not available.", "Нет корма для питомца: {}."),
    ("You can't put this item on.", "Этот предмет нельзя надеть."),
    ("From", "От"),
    ("Status (Alt+A)", "Характеристики (Alt+A)"),
    ("Equipment (Alt+Q)", "Экипировка (Alt+Q)"),
    ("Inventory (Alt+E)", "Инвентарь (Alt+E)"),
    ("Skills (Alt+S)", "Умения (Alt+S)"),
    ("Party (Alt+Z)", "Группа (Alt+Z)"),
    ("Guild (Alt+G)", "Гильдия (Alt+G)"),
    ("Battlegrounds", "Поля битв"),
    ("Quests (Alt+U)", "Задания (Alt+U)"),
    ("World map (Ctrl+~)", "Карта мира (Ctrl+~)"),
    ("Navigation", "Навигация"),
    ("Options (Alt+O)", "Настройки (Alt+O)"),
    ("Bank", "Банк"),
    ("Replay", "Запись"),
    ("Mail", "Почта"),
    ("Achievements", "Достижения"),
    ("Tips", "Подсказки"),
    ("Cash shop", "Кэш-магазин"),
    ("Shortcut settings", "Горячие клавиши"),
    ("Attendance", "Посещаемость"),
    ("Adventurer agency", "Агентство приключенцев"),
    ("Reputation", "Репутация"),
    ("Adventure guide", "Путеводитель"),
    ("Probabilities", "Вероятности"),
];
