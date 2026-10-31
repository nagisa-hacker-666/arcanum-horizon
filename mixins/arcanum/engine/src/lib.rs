//! Game logic of **Arcanum**, a magic and spellcraft mod for
//! [Horizon](https://github.com/horizon-rust/horizon).
//!
//! This crate is the whole of Arcanum that can be reasoned about without the
//! game: the six schools, the modular spell builder, the mana / resonance /
//! cooldown economy, the RPG progression, and the translation of a cast into a
//! list of vanilla commands the authoritative server runs.
//!
//! It has no dependency on Horizon and no dependency at all, so it is tested
//! with a plain `cargo test`. The mod's bundle
//! (`mixins/arcanum/horizon-mixin.toml`) stages it as a `[[crate]]` next to a
//! handful of hook helpers that translate `Game` into calls on this crate; see
//! `docs/` for the split between what is verified here and what only the game
//! can verify.
//!
//! ```
//! use arcanum_engine::{Limits, SpellBuilder, Element, Shape, Effect, Modifier};
//!
//! let spell = SpellBuilder::new(&Limits::default(), "pyro_fireball", "Fireball", Element::Pyro, Shape::Projectile, Effect::Damage)
//!     .modifier(Modifier::Explosive)
//!     .build()
//!     .expect("a legal spell");
//! assert!(spell.cost > 0.0 && spell.power > 0.0 && spell.range > 0.0);
//! ```

pub mod balance;
pub mod cast;
pub mod codec;
pub mod cooldown;
pub mod effects;
pub mod elements;
pub mod mana;
pub mod modifiers;
pub mod net;
pub mod player;
pub mod progression;
pub mod registry;
pub mod shapes;
pub mod spell;
pub mod spell_builder;

pub use balance::{Limits, MAX_LEVEL};
pub use cast::{resolve, CastError, EffectPlan};
pub use cooldown::Cooldowns;
pub use effects::{Effect, RenderQuality};
pub use elements::Element;
pub use mana::{Mana, Resonance};
pub use modifiers::Modifier;
pub use player::PlayerState;
pub use progression::{Progression, Stats, Talent, TALENTS};
pub use registry::SpellRegistry;
pub use shapes::Shape;
pub use spell::{Spell, SpellSpec};
pub use spell_builder::{BuildError, SpellBuilder};

/// The mod id: the bundle folder, the registry id, the settings folder and the
/// channel prefix of mod messages.
pub const MOD_ID: &str = "arcanum";

/// The version of the mod, as `horizon-mixin.toml` and `horizon-native.json`
/// carry it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The channel a client casts on.
pub const CHANNEL_CAST: &str = "arcanum:cast";

/// The channel the server pushes a player's state on.
pub const CHANNEL_STATE: &str = "arcanum:state";
