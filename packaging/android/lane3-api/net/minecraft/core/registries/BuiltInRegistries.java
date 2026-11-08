// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core.registries;

import dev.horizon.android.lane3.SimpleRegistry;
import net.minecraft.core.DefaultedRegistry;
import net.minecraft.world.item.Item;
import net.minecraft.world.level.block.Block;

public final class BuiltInRegistries {
    private BuiltInRegistries() {}

    public static final DefaultedRegistry<Block> BLOCK = new SimpleRegistry<>("minecraft:air", Block::new);
    public static final DefaultedRegistry<Item> ITEM = new SimpleRegistry<>("minecraft:air", Item::new);
}
