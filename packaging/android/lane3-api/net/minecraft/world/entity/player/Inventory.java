// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity.player;

/** A player's inventory: lane 3 knows its player (its items are Horizon's). */
public class Inventory implements net.minecraft.world.Container {
    public final Player player;

    public Inventory(Player player) {
        this.player = player;
    }

    @Override
    public int getContainerSize() {
        return 41;
    }

    public void placeItemBackInInventory(net.minecraft.world.item.ItemStack stack) {}
}
