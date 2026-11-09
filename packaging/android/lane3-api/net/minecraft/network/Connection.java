// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network;

/** A player's connection: lane 3's messages don't leave the game. */
public class Connection {
    public void disconnect(net.minecraft.network.chat.Component reason) {
        dev.horizon.android.lane3.Native.log("a mod would disconnect the player: " + (reason == null ? "" : reason.getString()));
    }
}
