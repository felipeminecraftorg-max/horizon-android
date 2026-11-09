// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.client.multiplayer;

/** The client's level: what a screen does there stays on the client (the server acts for real). */
public class ClientLevel extends net.minecraft.world.level.Level {
    public static final ClientLevel CURRENT = new ClientLevel();

    private ClientLevel() {
        super(true);
    }
}
