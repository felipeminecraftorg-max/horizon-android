// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.bus.api;

/** An event a listener can cancel: the game then does not do what it was about to. */
public interface ICancellableEvent {
    default void setCanceled(boolean canceled) {
        ((Event) this).horizonCanceled = canceled;
    }

    default boolean isCanceled() {
        return ((Event) this).horizonCanceled;
    }
}
