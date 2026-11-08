// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server;

/** Work to run at a server tick. */
public class TickTask implements Runnable {
    private final int tick;
    private final Runnable runnable;

    public TickTask(int tick, Runnable runnable) {
        this.tick = tick;
        this.runnable = runnable;
    }

    public int getTick() {
        return tick;
    }

    @Override
    public void run() {
        runnable.run();
    }
}
