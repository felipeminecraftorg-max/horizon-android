// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core.registries;

import dev.horizon.android.lane3.SimpleRegistry;
import net.minecraft.core.DefaultedRegistry;
import net.minecraft.world.item.Item;
import net.minecraft.world.level.block.Block;

public final class BuiltInRegistries {
    private BuiltInRegistries() {}

    public static final DefaultedRegistry<Block> BLOCK = new SimpleRegistry<Block>(Registries.BLOCK, "minecraft:air", Block::new);
    public static final DefaultedRegistry<Item> ITEM = new SimpleRegistry<Item>(Registries.ITEM, "minecraft:air", Item::new);
    public static final net.minecraft.core.Registry<net.minecraft.sounds.SoundEvent> SOUND_EVENT = new SimpleRegistry<net.minecraft.sounds.SoundEvent>(
            Registries.SOUND_EVENT, "minecraft:intentionally_empty",
            id -> net.minecraft.sounds.SoundEvent.createVariableRangeEvent(net.minecraft.resources.ResourceLocation.parse(id)));
    // The registries a mod registers into by the registry itself (DeferredRegister.create(registry, ...)):
    // what lane 3 doesn't make stands for nothing.
    public static final net.minecraft.core.Registry<Object> BLOCK_ENTITY_TYPE = nothing(Registries.BLOCK_ENTITY_TYPE);
    public static final net.minecraft.core.Registry<Object> MENU = nothing(Registries.MENU);
    public static final net.minecraft.core.Registry<Object> MOB_EFFECT = nothing(Registries.MOB_EFFECT);
    public static final net.minecraft.core.Registry<Object> POTION = nothing(Registries.POTION);
    public static final net.minecraft.core.Registry<Object> CREATIVE_MODE_TAB = nothing(Registries.CREATIVE_MODE_TAB);
    public static final net.minecraft.core.Registry<Object> ATTRIBUTE = nothing(Registries.ATTRIBUTE);
    public static final net.minecraft.core.Registry<Object> ARMOR_MATERIAL = nothing(Registries.ARMOR_MATERIAL);
    public static final net.minecraft.core.Registry<Object> RECIPE_TYPE = nothing(Registries.RECIPE_TYPE);
    public static final net.minecraft.core.Registry<Object> RECIPE_SERIALIZER = nothing(Registries.RECIPE_SERIALIZER);
    public static final net.minecraft.core.Registry<Object> GAME_EVENT = nothing(Registries.GAME_EVENT);
    public static final net.minecraft.core.Registry<Object> FEATURE = nothing(Registries.FEATURE);
    public static final net.minecraft.core.Registry<Object> DATA_COMPONENT_TYPE = nothing(Registries.DATA_COMPONENT_TYPE);
    public static final DefaultedRegistry<Object> ENTITY_TYPE = nothing(Registries.ENTITY_TYPE);
    public static final DefaultedRegistry<Object> FLUID = nothing(Registries.FLUID);

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static SimpleRegistry<Object> nothing(net.minecraft.resources.ResourceKey key) {
        return new SimpleRegistry<Object>(key, "minecraft:empty", id -> null);
    }

    public static final net.minecraft.core.Registry<net.minecraft.core.particles.ParticleType<?>> PARTICLE_TYPE =
            new SimpleRegistry<net.minecraft.core.particles.ParticleType<?>>(
                    Registries.PARTICLE_TYPE, "minecraft:smoke", net.minecraft.core.particles.SimpleParticleType::new);
}
