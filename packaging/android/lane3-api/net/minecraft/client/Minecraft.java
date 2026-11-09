// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client;

/** The client, as a mod's screen reaches it: its player and the screen open. */
public class Minecraft {
    private static final Minecraft INSTANCE = new Minecraft();

    public final net.minecraft.client.player.LocalPlayer player = net.minecraft.client.player.LocalPlayer.CURRENT;
    public final net.minecraft.client.multiplayer.ClientLevel level = net.minecraft.client.multiplayer.ClientLevel.CURRENT;
    public final net.minecraft.client.gui.Font font = new net.minecraft.client.gui.Font();
    public net.minecraft.client.gui.screens.Screen screen;

    public static Minecraft getInstance() {
        return INSTANCE;
    }

    public void setScreen(net.minecraft.client.gui.screens.Screen next) {
        if (next == null) {
            if (screen != null) dev.horizon.android.lane3.Neo.closeScreen(true);
            return;
        }
        screen = next;
    }

    public boolean isSameThread() {
        return true;
    }

    public void execute(Runnable work) {
        work.run();
    }
}
