// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.inventory;

/** Where a menu's block is. */
public interface ContainerLevelAccess {
    ContainerLevelAccess NULL = new ContainerLevelAccess() {};

    static ContainerLevelAccess create(net.minecraft.world.level.Level level, net.minecraft.core.BlockPos pos) {
        return new ContainerLevelAccess() {};
    }
}
