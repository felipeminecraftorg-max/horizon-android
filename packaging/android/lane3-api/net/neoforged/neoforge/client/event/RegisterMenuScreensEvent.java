// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.client.event;

/** Mods give their menu types their screens. */
public class RegisterMenuScreensEvent extends net.neoforged.bus.api.Event {
    public <M extends net.minecraft.world.inventory.AbstractContainerMenu, U extends net.minecraft.client.gui.screens.Screen
            & net.minecraft.client.gui.screens.inventory.MenuAccess<M>> void register(
            net.minecraft.world.inventory.MenuType<? extends M> type, net.minecraft.client.gui.screens.MenuScreens.ScreenConstructor<M, U> constructor) {
        dev.horizon.android.lane3.Neo.screen(type, constructor);
    }
}
