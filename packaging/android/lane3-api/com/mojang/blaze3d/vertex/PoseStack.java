// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package com.mojang.blaze3d.vertex;

/** Where a screen draws: lane 3 keeps its moves (not its turns or scales). */
public class PoseStack {
    private final java.util.ArrayDeque<double[]> stack = new java.util.ArrayDeque<>();
    private double x;
    private double y;

    public void pushPose() {
        stack.push(new double[] {x, y});
    }

    public void popPose() {
        double[] top = stack.poll();
        if (top != null) {
            x = top[0];
            y = top[1];
        }
    }

    public void translate(double dx, double dy, double dz) {
        x += dx;
        y += dy;
    }

    public void translate(float dx, float dy, float dz) {
        x += dx;
        y += dy;
    }

    public void scale(float sx, float sy, float sz) {}

    /** The move so far. */
    public double horizonX() {
        return x;
    }

    public double horizonY() {
        return y;
    }
}
