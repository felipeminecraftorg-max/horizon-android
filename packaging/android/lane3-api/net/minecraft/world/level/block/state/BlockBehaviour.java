// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state;

import net.minecraft.core.BlockPos;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.Level;

/** What a block does: a mod's block overrides what it does otherwise. */
public abstract class BlockBehaviour {
    protected final Properties properties;

    protected BlockBehaviour(Properties properties) {
        this.properties = properties == null ? Properties.of() : properties;
    }

    /** An entity is inside the block's space. */
    protected void entityInside(BlockState state, Level level, BlockPos pos, Entity entity) {}

    /** How a block is made: lane 3 keeps what it needs of it. */
    public static class Properties {
        public static Properties of() {
            return new Properties();
        }
    }
}
