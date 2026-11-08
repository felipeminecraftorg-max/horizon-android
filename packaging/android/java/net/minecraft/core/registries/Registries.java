// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core.registries;

import net.minecraft.core.Registry;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

/** The names of the game's registries, as keys. */
@SuppressWarnings({"rawtypes", "unchecked"})
public final class Registries {
    public static final ResourceKey ACTIVITY = key("activity");
    public static final ResourceKey ATTRIBUTE = key("attribute");
    public static final ResourceKey BANNER_PATTERN = key("banner_pattern");
    public static final ResourceKey BIOME = key("worldgen/biome");
    public static final ResourceKey BLOCK = key("block");
    public static final ResourceKey BLOCK_ENTITY_TYPE = key("block_entity_type");
    public static final ResourceKey BLOCK_PREDICATE_TYPE = key("block_predicate_type");
    public static final ResourceKey CAT_VARIANT = key("cat_variant");
    public static final ResourceKey CHAT_TYPE = key("chat_type");
    public static final ResourceKey CHUNK_GENERATOR = key("chunk_generator");
    public static final ResourceKey CHUNK_STATUS = key("chunk_status");
    public static final ResourceKey COMMAND_ARGUMENT_TYPE = key("command_argument_type");
    public static final ResourceKey CONFIGURED_CARVER = key("worldgen/configured_carver");
    public static final ResourceKey CONFIGURED_FEATURE = key("worldgen/configured_feature");
    public static final ResourceKey CREATIVE_MODE_TAB = key("creative_mode_tab");
    public static final ResourceKey CUSTOM_STAT = key("custom_stat");
    public static final ResourceKey DAMAGE_TYPE = key("damage_type");
    public static final ResourceKey DATA_COMPONENT_TYPE = key("data_component_type");
    public static final ResourceKey DECORATED_POT_PATTERN = key("decorated_pot_pattern");
    public static final ResourceKey DENSITY_FUNCTION = key("worldgen/density_function");
    public static final ResourceKey DIMENSION = key("dimension");
    public static final ResourceKey DIMENSION_TYPE = key("dimension_type");
    public static final ResourceKey ENCHANTMENT = key("enchantment");
    public static final ResourceKey ENCHANTMENT_EFFECT_COMPONENT_TYPE = key("enchantment_effect_component_type");
    public static final ResourceKey ENTITY_TYPE = key("entity_type");
    public static final ResourceKey FEATURE = key("feature");
    public static final ResourceKey FLOAT_PROVIDER_TYPE = key("float_provider_type");
    public static final ResourceKey FLUID = key("fluid");
    public static final ResourceKey FROG_VARIANT = key("frog_variant");
    public static final ResourceKey GAME_EVENT = key("game_event");
    public static final ResourceKey HEIGHT_PROVIDER_TYPE = key("height_provider_type");
    public static final ResourceKey INSTRUMENT = key("instrument");
    public static final ResourceKey INT_PROVIDER_TYPE = key("int_provider_type");
    public static final ResourceKey ITEM = key("item");
    public static final ResourceKey JUKEBOX_SONG = key("jukebox_song");
    public static final ResourceKey LEVEL_STEM = key("level_stem");
    public static final ResourceKey LOOT_CONDITION_TYPE = key("loot_condition_type");
    public static final ResourceKey LOOT_FUNCTION_TYPE = key("loot_function_type");
    public static final ResourceKey LOOT_POOL_ENTRY_TYPE = key("loot_pool_entry_type");
    public static final ResourceKey MEMORY_MODULE_TYPE = key("memory_module_type");
    public static final ResourceKey MENU = key("menu");
    public static final ResourceKey MOB_EFFECT = key("mob_effect");
    public static final ResourceKey NOISE = key("worldgen/noise");
    public static final ResourceKey NOISE_SETTINGS = key("worldgen/noise_settings");
    public static final ResourceKey PAINTING_VARIANT = key("painting_variant");
    public static final ResourceKey PARTICLE_TYPE = key("particle_type");
    public static final ResourceKey PLACED_FEATURE = key("worldgen/placed_feature");
    public static final ResourceKey POINT_OF_INTEREST_TYPE = key("point_of_interest_type");
    public static final ResourceKey POS_RULE_TEST = key("pos_rule_test");
    public static final ResourceKey POTION = key("potion");
    public static final ResourceKey PROCESSOR_LIST = key("worldgen/processor_list");
    public static final ResourceKey RECIPE_SERIALIZER = key("recipe_serializer");
    public static final ResourceKey RECIPE_TYPE = key("recipe_type");
    public static final ResourceKey RULE_TEST = key("rule_test");
    public static final ResourceKey SCHEDULE = key("schedule");
    public static final ResourceKey SENSOR_TYPE = key("sensor_type");
    public static final ResourceKey SOUND_EVENT = key("sound_event");
    public static final ResourceKey STAT_TYPE = key("stat_type");
    public static final ResourceKey STRUCTURE = key("worldgen/structure");
    public static final ResourceKey STRUCTURE_PIECE = key("structure_piece");
    public static final ResourceKey STRUCTURE_PLACEMENT = key("structure_placement");
    public static final ResourceKey STRUCTURE_POOL_ELEMENT = key("structure_pool_element");
    public static final ResourceKey STRUCTURE_PROCESSOR = key("structure_processor");
    public static final ResourceKey STRUCTURE_SET = key("worldgen/structure_set");
    public static final ResourceKey STRUCTURE_TYPE = key("structure_type");
    public static final ResourceKey TEMPLATE_POOL = key("worldgen/template_pool");
    public static final ResourceKey TRIM_MATERIAL = key("trim_material");
    public static final ResourceKey TRIM_PATTERN = key("trim_pattern");
    public static final ResourceKey VILLAGER_PROFESSION = key("villager_profession");
    public static final ResourceKey VILLAGER_TYPE = key("villager_type");
    public static final ResourceKey WOLF_VARIANT = key("wolf_variant");

    private Registries() {}

    private static ResourceKey key(String path) {
        return ResourceKey.createRegistryKey(ResourceLocation.withDefaultNamespace(path));
    }

    public static ResourceLocation elementsDirPath(ResourceKey<? extends Registry<?>> key) {
        return key.location();
    }
}
