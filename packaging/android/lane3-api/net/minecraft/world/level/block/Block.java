// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block;

import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.block.state.BlockState;

/** A block by its id: what it does is Horizon's (lane 1 carried a mod's own). */
public class Block extends net.minecraft.world.level.block.state.BlockBehaviour {
    String id;

    public Block(String id) {
        super(null);
        this.id = id;
    }

    /** A mod's block: its id is the one it is registered under (lane 3 sets it). */
    public Block(net.minecraft.world.level.block.state.BlockBehaviour.Properties properties) {
        super(properties);
    }

    /** The id lane 3 registered this block under. */
    public final void horizonId(String id) {
        if (this.id == null) this.id = id;
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
