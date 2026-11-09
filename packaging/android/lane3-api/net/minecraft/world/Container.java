// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world;

/** Item slots. */
public interface Container {
    default int getContainerSize() {
        return 0;
    }

    default net.minecraft.world.item.ItemStack getItem(int slot) {
        return net.minecraft.world.item.ItemStack.EMPTY;
    }

    default void setItem(int slot, net.minecraft.world.item.ItemStack stack) {}

    default void setChanged() {}

    default boolean stillValid(net.minecraft.world.entity.player.Player player) {
        return true;
    }
}
