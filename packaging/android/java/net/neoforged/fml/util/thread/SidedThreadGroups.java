// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml.util.thread;

import net.neoforged.fml.LogicalSide;

/** The server's and the client's thread groups: lane 3 runs the server's thread in {@code SERVER}. */
public final class SidedThreadGroups {
    public static final SidedThreadGroup CLIENT = new SidedThreadGroup(LogicalSide.CLIENT);
    public static final SidedThreadGroup SERVER = new SidedThreadGroup(LogicalSide.SERVER);

    private SidedThreadGroups() {}
}
