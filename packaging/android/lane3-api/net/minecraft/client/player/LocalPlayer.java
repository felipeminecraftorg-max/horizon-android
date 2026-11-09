// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.player;

/** The client's own player: a mod's screen acts on the client side through it. */
public class LocalPlayer extends AbstractClientPlayer {
    public static final LocalPlayer CURRENT = new LocalPlayer();

    @Override
    public net.minecraft.world.level.Level level() {
        return net.minecraft.client.multiplayer.ClientLevel.CURRENT;
    }

    public void closeContainer() {
        dev.horizon.android.lane3.Neo.closeScreen(true);
    }
}
