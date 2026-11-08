// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml.util.thread;

import net.neoforged.fml.LogicalSide;

/** The threads of one side: lane 3 calls a mod's code on the server's. */
public final class SidedThreadGroup extends ThreadGroup {
    private final LogicalSide side;

    public SidedThreadGroup(LogicalSide side) {
        super(side == LogicalSide.SERVER ? "SERVER" : "CLIENT");
        this.side = side;
    }

    public LogicalSide getSide() {
        return side;
    }
}
