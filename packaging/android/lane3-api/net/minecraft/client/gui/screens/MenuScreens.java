// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.gui.screens;

/** Which screen each menu type has. */
public final class MenuScreens {
    private MenuScreens() {}

    @FunctionalInterface
    public interface ScreenConstructor<T extends net.minecraft.world.inventory.AbstractContainerMenu, U extends Screen> {
        U create(T menu, net.minecraft.world.entity.player.Inventory inventory, net.minecraft.network.chat.Component title);
    }
}
