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
    public static final net.minecraft.core.Registry<net.minecraft.core.particles.ParticleType<?>> PARTICLE_TYPE =
            new SimpleRegistry<net.minecraft.core.particles.ParticleType<?>>(
                    Registries.PARTICLE_TYPE, "minecraft:smoke", net.minecraft.core.particles.SimpleParticleType::new);
}
