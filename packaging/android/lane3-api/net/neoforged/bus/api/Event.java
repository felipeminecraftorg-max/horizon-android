// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.bus.api;

/** Something that happened, posted to the listeners of its type. */
public abstract class Event {
    boolean horizonCanceled;

    protected Event() {}
}
