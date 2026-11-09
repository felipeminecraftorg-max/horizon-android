// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world;

/** What makes a menu for a player. */
@FunctionalInterface
public interface MenuConstructor {
    net.minecraft.world.inventory.AbstractContainerMenu createMenu(int containerId, net.minecraft.world.entity.player.Inventory inventory,
            net.minecraft.world.entity.player.Player player);
}
