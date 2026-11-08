// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block;

import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.block.state.BlockState;

/** A block by its id: what it does is Horizon's (lane 1 carried a mod's own). */
public class Block {
    final String id;

    public Block(String id) {
        this.id = id;
    }

    public BlockState defaultBlockState() {
        return BlockState.of(id);
    }

    public String getDescriptionId() {
        int colon = id.indexOf(':');
        return "block." + id.substring(0, colon) + "." + id.substring(colon + 1);
    }

    public String id() {
        return id;
    }

    public static Block byId(String id) {
        return BuiltInRegistries.BLOCK.get(net.minecraft.resources.ResourceLocation.parse(id));
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Block b && b.id.equals(id);
    }

    @Override
    public int hashCode() {
        return id.hashCode();
    }

    @Override
    public String toString() {
        return "Block{" + id + "}";
    }
}
