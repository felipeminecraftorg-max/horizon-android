// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.screens.inventory;

/** A screen of a menu. */
public interface MenuAccess<T extends net.minecraft.world.inventory.AbstractContainerMenu> {
    T getMenu();
}
