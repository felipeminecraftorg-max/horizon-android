// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml.event.lifecycle;

/** The client's setup: work queued for it runs at once. */
public class FMLClientSetupEvent extends net.neoforged.bus.api.Event {
    public java.util.concurrent.CompletableFuture<Void> enqueueWork(Runnable work) {
        work.run();
        return java.util.concurrent.CompletableFuture.completedFuture(null);
    }
}
