use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyState { Idle, Chase, Dead }

// ── Munición ─────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AmmoType {
    Light,   // balas ligeras
    Medium,  // balas medianas
    Heavy,   // balas pesadas
    Shell,   // cartuchos escopeta
    Rocket,  // cohetes
}

impl AmmoType {
    pub const COUNT: usize = 5;
    pub fn index(self) -> usize {
        match self {
            AmmoType::Light  => 0,
            AmmoType::Medium => 1,
            AmmoType::Heavy  => 2,
            AmmoType::Shell  => 3,
            AmmoType::Rocket => 4,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            AmmoType::Light  => "Ligeras",
            AmmoType::Medium => "Medianas",
            AmmoType::Heavy  => "Pesadas",
            AmmoType::Shell  => "Cartuchos",
            AmmoType::Rocket => "Cohetes",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rarity {
    Common,       // Común
    Uncommon,     // Poco común
    Rare,         // Rara
    Epic,         // Épica
    Legendary,    // Legendaria
}

impl Rarity {
    pub fn damage_mult(self) -> f32 {
        match self {
            Rarity::Common    => 1.00,
            Rarity::Uncommon  => 1.10,
            Rarity::Rare      => 1.22,
            Rarity::Epic      => 1.36,
            Rarity::Legendary => 1.52,
        }
    }
    pub fn reload_mult(self) -> f32 {
        // Multiplicador sobre el tiempo de recarga (más raro ==> menor tiempo)
        match self {
            Rarity::Common    => 1.00,
            Rarity::Uncommon  => 0.95,
            Rarity::Rare      => 0.88,
            Rarity::Epic      => 0.80,
            Rarity::Legendary => 0.70,
        }
    }
    pub fn color(self) -> Color {
        match self {
            Rarity::Common    => Color::LIGHTGRAY,
            Rarity::Uncommon  => Color::LIME,
            Rarity::Rare      => Color::BLUE,
            Rarity::Epic      => Color::PURPLE,
            Rarity::Legendary => Color::GOLD,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common    => "Común",
            Rarity::Uncommon  => "Poco común",
            Rarity::Rare      => "Rara",
            Rarity::Epic      => "Épica",
            Rarity::Legendary => "Legendaria",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WeaponType {
    Pistol,
    SMG,
    Rifle,
    Shotgun,
    RocketLauncher,
}

impl WeaponType {
    pub fn name(self) -> &'static str {
        match self {
            WeaponType::Pistol => "Pistola",
            WeaponType::SMG => "SMG",
            WeaponType::Rifle => "Rifle",
            WeaponType::Shotgun => "Escopeta",
            WeaponType::RocketLauncher => "Lanzacohetes",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Weapon {
    pub wtype: WeaponType,      // <- tipo
    pub name: &'static str,     // nombre visible
    pub damage: i32,            // daño BASE
    pub fire_interval: f32,
    pub mag_size: i32,
    pub reload_time: f32,       // recarga BASE
    pub ammo_type: AmmoType,
    pub rarity: Rarity,         // <- rareza
}

impl Weapon {
    pub fn effective_damage(&self) -> i32 {
        ((self.damage as f32) * self.rarity.damage_mult()).round() as i32
    }
    pub fn effective_reload(&self) -> f32 {
        self.reload_time * self.rarity.reload_mult()
    }
}

pub struct Enemy {
    pub x: f32, pub y: f32,
    pub hp: i32,
    pub speed: f32,
    pub state: EnemyState,
    pub color: Color,
    pub weapon: Weapon,
    pub weapon_cd: f32,
    pub flash_timer: f32,
    pub death_anim_t: f32,
}
impl Enemy { pub fn is_alive(&self) -> bool { self.state != EnemyState::Dead && self.hp > 0 } }

// ── Pickups ─────────────────────────────────────────────
#[derive(Clone, Copy)]
pub enum PickupKind {
    // Salud (legacy, los seguimos aceptando para spawns del mapa/cofres)
    HealthSmall,
    HealthBig,
    ShieldSmall,
    ShieldBig,
    // Munición
    Ammo { ammo: AmmoType, amount: i32 },
    // Arma con rareza
    Weapon { wtype: WeaponType, rarity: Rarity },
    // 🔹 Ítem exacto (arma o consumible), con stack opcional (para drops)
    Item { item: Item, count: i32 },
}

pub struct Pickup {
    pub x: f32,
    pub y: f32,
    pub kind: PickupKind,
    pub radius: f32,
    pub color: Color,
    pub spawn_lock: f32, // para evitar recoger en el mismo frame del spawn
}

pub struct Player {
    pub x: f32, pub y: f32, pub angle: f32,
    pub hp: i32,
    pub shield: i32,

    // Reserva de munición por tipo (igual que antes)
    pub ammo_reserve: [i32; AmmoType::COUNT],

    // 5 slots unificados (armas/consumibles) y selección actual
    pub slots: [Option<SlotItem>; 5],
    pub selected: Option<usize>,

    pub punch_cd: f32,

}


#[derive(Clone, Copy)]
pub struct DrawnSprite { pub start_x: i32, pub end_x: i32, pub depth: f32, pub idx: usize }

// ── Decoraciones y cofres (ya los tienes) ─────────────────
#[derive(Clone, Copy)]
pub enum DecoKind { Blocking, Ghost }
pub struct Decoration { pub x: f32, pub y: f32, pub radius: f32, pub color: Color, pub kind: DecoKind }
impl Decoration { pub fn is_blocking(&self) -> bool { matches!(self.kind, DecoKind::Blocking) } }

pub struct Chest {
    pub x: f32, pub y: f32,
    pub radius: f32,
    pub opened: bool,
    pub color_closed: Color,
    pub color_opened: Color,
}
impl Chest { pub fn is_blocking(&self) -> bool { true } }


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConsumableType {
    HealthSmall, // cura 20 hasta 75
    HealthBig,   // cura 100 (cap 100)
    ShieldSmall, // +25 hasta 50
    ShieldBig,   // +50 (cap 100)
}

impl ConsumableType {
    pub fn name(self) -> &'static str {
        match self {
            ConsumableType::HealthSmall => "Vida menor",
            ConsumableType::HealthBig   => "Vida mayor",
            ConsumableType::ShieldSmall => "Escudo menor",
            ConsumableType::ShieldBig   => "Escudo mayor",
        }
    }
}

/// Estado runtime para un arma equipada en un slot
#[derive(Clone, Copy)]
pub struct WeaponState {
    pub ammo_in_mag: i32,
    pub weapon_cd: f32,
    pub reloading: bool,
    pub reload_cd: f32,
}

/// Un ítem que puede ir en un slot: arma (con su estado) o consumible
#[derive(Clone, Copy)]
pub enum Item {
    Weapon(Weapon, WeaponState),
    Consumable(ConsumableType),
}

/// Slot (1 de 5)
#[derive(Clone, Copy)]
pub struct SlotItem {
    pub item: Item,
    pub count: i32, // armas: 1; consumibles: stack
    pub cd: f32,    // ⬅️ lo usaremos como "tiempo restante de uso/canalización"
    pub using: bool, // ⬅️ NUEVO: ¿está en uso ahora mismo?
}

