package rsmod

import java.io.File

object PathFinder {
    init {
        System.load(File("./lib/rsmod.dll").absolutePath)
    }

    @JvmStatic
    external fun findPath(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        srcSize: Int,
        destWidth: Int,
        destHeight: Int,
        angle: Int,
        shape: Int,
        moveNear: Boolean,
        blockAccessFlags: Int,
        maxWaypoints: Int,
        collision: Int
    ): IntArray

    @JvmStatic
    external fun findNaivePath(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        srcWidth: Int,
        srcHeight: Int,
        destWidth: Int,
        destHeight: Int,
        extraFlag: Int,
        collision: Int
    ): IntArray

    @JvmStatic
    external fun changeFloor(
        x: Int,
        z: Int,
        y: Int,
        add: Boolean
    )

    @JvmStatic
    external fun changeLoc(
        x: Int,
        z: Int,
        y: Int,
        width: Int,
        length: Int,
        blockrange: Boolean,
        breakroutefinding: Boolean,
        add: Boolean
    )

    @JvmStatic
    external fun changeNpc(
        x: Int,
        z: Int,
        y: Int,
        size: Int,
        add: Boolean
    )

    @JvmStatic
    external fun changePlayer(
        x: Int,
        z: Int,
        y: Int,
        size: Int,
        add: Boolean
    )

    @JvmStatic
    external fun changeRoof(
        x: Int,
        z: Int,
        y: Int,
        add: Boolean
    )

    @JvmStatic
    external fun changeWall(
        x: Int,
        z: Int,
        y: Int,
        angle: Int,
        shape: Int,
        blockrange: Boolean,
        breakroutefinding: Boolean,
        add: Boolean
    )

    @JvmStatic
    external fun changeWallStraight(
        x: Int,
        z: Int,
        y: Int,
        angle: Int,
        blockrange: Boolean,
        breakroutefinding: Boolean,
        add: Boolean
    )

    @JvmStatic
    external fun changeWallCorner(
        x: Int,
        z: Int,
        y: Int,
        angle: Int,
        blockrange: Boolean,
        breakroutefinding: Boolean,
        add: Boolean
    )

    @JvmStatic
    external fun changeWallL(
        x: Int,
        z: Int,
        y: Int,
        angle: Int,
        blockrange: Boolean,
        breakroutefinding: Boolean,
        add: Boolean
    )

    @JvmStatic
    external fun allocateIfAbsent(
        x: Int,
        z: Int,
        y: Int
    )

    @JvmStatic
    external fun deallocateIfPresent(
        x: Int,
        z: Int,
        y: Int
    )

    @JvmStatic
    external fun isZoneAllocated(
        x: Int,
        z: Int,
        y: Int
    ): Boolean

    @JvmStatic
    external fun isFlagged(
        x: Int,
        z: Int,
        y: Int,
        masks: Int
    ): Boolean

    @JvmStatic
    external fun canTravel(
        y: Int,
        x: Int,
        z: Int,
        offsetX: Int,
        offsetZ: Int,
        size: Int,
        extraFlag: Int,
        collision: Int
    ): Boolean

    @JvmStatic
    external fun hasLineOfSight(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        srcWidth: Int,
        srcHeight: Int,
        destWidth: Int,
        destHeight: Int,
        extraFlag: Int
    ): Boolean

    @JvmStatic
    external fun hasLineOfWalk(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        srcWidth: Int,
        srcHeight: Int,
        destWidth: Int,
        destHeight: Int,
        extraFlag: Int
    ): Boolean

    @JvmStatic
    external fun lineOfSight(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        srcWidth: Int,
        srcHeight: Int,
        destWidth: Int,
        destHeight: Int,
        extraFlag: Int
    ): IntArray

    @JvmStatic
    external fun lineOfWalk(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        srcWidth: Int,
        srcHeight: Int,
        destWidth: Int,
        destHeight: Int,
        extraFlag: Int
    ): IntArray

    @JvmStatic
    external fun reached(
        y: Int,
        srcX: Int,
        srcZ: Int,
        destX: Int,
        destZ: Int,
        destWidth: Int,
        destHeight: Int,
        srcSize: Int,
        angle: Int,
        shape: Int,
        blockAccessFlags: Int
    ): Boolean

    @JvmStatic
    external fun locShapeLayer(
        shape: Int
    ): Int
}