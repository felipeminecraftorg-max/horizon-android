// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.registries;

import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

/** NeoForge's own registries, by name. */
@SuppressWarnings("rawtypes")
public final class NeoForgeRegistries {
    private NeoForgeRegistries() {}

    public static final class Keys {
        public static final ResourceKey ATTACHMENT_TYPES = key("attachment_types");
        public static final ResourceKey BIOME_MODIFIER_SERIALIZERS = key("biome_modifier_serializers");
        public static final ResourceKey STRUCTURE_MODIFIER_SERIALIZERS = key("structure_modifier_serializers");
        public static final ResourceKey FLUID_TYPES = key("fluid_type");
        public static final ResourceKey GLOBAL_LOOT_MODIFIER_SERIALIZERS = key("global_loot_modifier_serializers");
        public static final ResourceKey CONDITION_CODECS = key("condition_codecs");
        public static final ResourceKey INGREDIENT_TYPES = key("ingredient_serializer");
        public static final ResourceKey ENTITY_DATA_SERIALIZERS = key("entity_data_serializers");
        public static final ResourceKey BIOME_MODIFIERS = key("biome_modifier");
        public static final ResourceKey STRUCTURE_MODIFIERS = key("structure_modifier");

        private Keys() {}

        private static ResourceKey key(String path) {
            return ResourceKey.createRegistryKey(ResourceLocation.fromNamespaceAndPath("neoforge", path));
        }
    }
}
