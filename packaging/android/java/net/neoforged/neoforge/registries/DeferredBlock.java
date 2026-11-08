// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.registries;

import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.block.Block;

/** A block a mod registered. */
public class DeferredBlock<T extends Block> extends DeferredHolder<Block, T> {
    protected DeferredBlock(ResourceKey<Block> key) {
        super(key);
    }

    public static <T extends Block> DeferredBlock<T> createBlock(ResourceKey<Block> key) {
        return new DeferredBlock<>(key);
    }
}
