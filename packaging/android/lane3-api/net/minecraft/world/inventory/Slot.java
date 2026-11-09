// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.inventory;

import net.minecraft.world.item.ItemStack;

/** A slot of a menu, where its screen draws it. */
public class Slot {
    public final net.minecraft.world.Container container;
    private final int slot;
    public int index;
    public final int x;
    public final int y;

    /** A stand-in's slot (one of the game's kinds a mod extends). */
    protected Slot() {
        this(null, 0, 0, 0);
    }

    public Slot(net.minecraft.world.Container container, int slot, int x, int y) {
        this.container = container;
        this.slot = slot;
        this.x = x;
        this.y = y;
    }

    public ItemStack getItem() {
        return container == null ? ItemStack.EMPTY : container.getItem(slot);
    }

    public boolean hasItem() {
        return !getItem().isEmpty();
    }

    public void set(ItemStack stack) {
        if (container != null) container.setItem(slot, stack);
    }

    public void setByPlayer(ItemStack stack) {
        set(stack);
    }

    public void setChanged() {}

    public boolean mayPlace(ItemStack stack) {
        return true;
    }

    public boolean mayPickup(net.minecraft.world.entity.player.Player player) {
        return true;
    }

    public int getMaxStackSize() {
        return 64;
    }

    public int getMaxStackSize(ItemStack stack) {
        return 64;
    }

    public void onTake(net.minecraft.world.entity.player.Player player, ItemStack stack) {}

    public void onQuickCraft(ItemStack a, ItemStack b) {}

    public int getContainerSlot() {
        return slot;
    }

    public boolean isActive() {
        return true;
    }
}
