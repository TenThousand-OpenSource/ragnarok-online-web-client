# Глоссарий русского перевода (Ragnarok Online)

Правила:
- Структуру JSON не менять: те же ключи (ID), у каждого `name` и `description` — массив с ТЕМ ЖЕ числом строк в том же порядке.
- Цветовые коды `^RRGGBB` (например `^0000CC`, `^000000`, `^777777`) оставлять на своих местах без изменений.
- Числа, проценты, знаки `+`/`-`, строки-разделители из подчёркиваний `_______` не менять.
- Не переводить аббревиатуры характеристик: HP, SP, AP, ATK, MATK, DEF, MDEF, HIT, FLEE, CRIT, ASPD, STR, AGI, VIT, INT, DEX, LUK, EXP, Lv.
- Имена собственные монстров и городов транслитерировать привычно для русских игроков RO (Poring — Поринг, Prontera — Пронтера, Payon — Паён, Geffen — Геффен, Morroc — Моррок, Alberta — Альберта, Izlude — Излюд, Aldebaran — Альдебаран).
- Текст должен быть естественным русским, кратким, как в игре. Без кавычек-ёлочек вокруг названий, без пояснений переводчика.

Термины:
- Novice — Новичок; Swordsman — Мечник; Magician/Mage — Маг; Archer — Лучник; Merchant — Торговец; Thief — Вор; Acolyte — Послушник
- Knight — Рыцарь; Wizard — Волшебник; Hunter — Охотник; Blacksmith — Кузнец; Assassin — Убийца; Priest — Священник; Crusader — Крестоносец; Sage — Мудрец; Bard — Бард; Dancer — Танцовщица; Alchemist — Алхимик; Rogue — Разбойник; Monk — Монах; Super Novice — Супер Новичок; Soul Linker — Медиум; Ninja — Ниндзя; Gunslinger — Стрелок; Taekwon — Тхэквондист
- Skill — умение; Passive — пассивное; Active — активное; Max Level — Макс. уровень; Skill Form — Вид умения; Target — Цель; Description — Описание; Type — Тип; Requirement — Требования
- Enemy — враг; Self — на себя; Ally/Party — союзник/группа; Ground — область на земле; 1 Character — 1 персонаж
- Physical — физическое; Magical — магическое; Supportive — поддержка; Recovery — восстановление
- Fire — Огонь; Water — Вода; Wind — Ветер; Earth — Земля; Holy — Святость; Shadow/Dark — Тьма; Ghost — Призрак; Undead — Нежить; Poison — Яд; Neutral — Нейтральный
- Hit(s) — удар(ов); SP Consumption — расход SP; Cast Time — время каста; Cooldown/After Cast Delay — задержка после каста; Duration — длительность; Range — дальность
- Weight — Вес; Heal — Лечение; Restorative — Восстанавливающий предмет; Collectible — Трофей; Attack — Атака; Defense — Защита; Weapon Level — Уровень оружия; Armor Level — Уровень брони; Location/Position — Место; Refineable — Заточка; Class — Класс; Property/Element — Стихия
- Potion — зелье; Herb — трава; Card — карта; Arrow — стрела; Zeny — зени

Дополнение для оружия и брони:
- Названия умений первых профессий брать из `translations/ru/skills-first-jobs.json` (поле `name`), названия расходников — из `translations/ru/items-0501-0700.json`, лута и материалов — из `items-0701-0900.json` и `items-0901-1099.json`, если предмет там есть.
- Типы оружия: Dagger — Кинжал; One-Handed Sword — Одноручный меч; Two-Handed Sword — Двуручный меч; One-Handed Spear — Одноручное копьё; Two-Handed Spear — Двуручное копьё; One-Handed Axe — Одноручный топор; Two-Handed Axe — Двуручный топор; Mace — Булава; Staff/Rod — Посох; Two-Handed Staff — Двуручный посох; Bow — Лук; Knuckle — Кастет; Musical Instrument — Музыкальный инструмент; Whip — Кнут; Book — Книга; Katar — Катар; Revolver — Револьвер; Rifle — Винтовка; Gatling Gun — Пулемёт; Shotgun — Дробовик; Grenade Launcher — Гранатомёт; Huuma Shuriken — Сюрикэн фума
- Броня: Armor — Броня; Shield — Щит; Garment — Накидка; Shoes/Footgear — Обувь; Accessory — Аксессуар; Headgear — Головной убор; Upper/Middle/Lower — Верх/Середина/Низ
- Метки: Attack — Атака; MATK — MATK; Defense — Защита; Weight — Вес; Weapon Level — Уровень оружия; Armor Level — Уровень брони; Base Level — Базовый уровень; Refineable/Refinable — Заточка (Yes — Да, No — Нет); Requirement — Требования; Property/Element — Стихия; Indestructible — Неразрушимый; Class — Класс; Position/Location — Место; Set Bonus — Бонус комплекта; Refine Level — Уровень заточки
- Вторые и высшие профессии: Lord Knight — Лорд-рыцарь; High Wizard — Верховный волшебник; Sniper — Снайпер; Whitesmith/Mastersmith — Мастер-кузнец; Assassin Cross — Ассасин Креста; High Priest — Первосвященник; Paladin — Паладин; Professor/Scholar — Профессор; Clown/Minstrel — Менестрель; Gypsy — Цыганка; Creator/Biochemist — Биохимик; Stalker — Сталкер; Champion — Чемпион; Star Gladiator — Звёздный гладиатор; Rune Knight — Рыцарь рун; Warlock — Чернокнижник; Ranger — Рейнджер; Mechanic — Механик; Guillotine Cross — Гильотинный крест; Arch Bishop — Архиепископ; Royal Guard — Королевский страж; Sorcerer — Колдун; Shadow Chaser — Теневой охотник; Sura — Сура; Genetic — Генетик; Wanderer — Странница; Rebellion — Мятежник; Kagerou/Oboro — Кагэро/Оборо; Summoner — Призыватель; Transcendent — перерождённые
- Статусы: Stun — Оглушение; Freeze — Заморозка; Stone Curse/Petrify — Окаменение; Sleep — Сон; Poison — Отравление; Curse — Проклятие; Silence — Молчание; Blind — Слепота; Bleeding — Кровотечение; Confuse/Chaos — Замешательство
- Расы и размеры: Demi-Human — Человекоподобный; Brute — Зверь; Plant — Растение; Insect — Насекомое; Fish — Рыба; Demon — Демон; Undead — Нежить; Formless — Бесформенный; Angel — Ангел; Dragon — Дракон; Small/Medium/Large — Маленький/Средний/Большой; Boss — Босс
