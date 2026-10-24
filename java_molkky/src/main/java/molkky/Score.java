package molkky;

final class Score {
    private final int value;

    public Score(Throw... recordedThrows) {
        if (recordedThrows == null) {
            throw new IllegalArgumentException("投てきの配列はnullにできません");
        }
        int total = 0;
        // 同じ内容の投てきも別の1回として数え、重複を除かず点数を合計する。
        for (Throw recordedThrow : recordedThrows) {
            if (recordedThrow == null) {
                throw new IllegalArgumentException("投てきはnullにできません");
            }
            total += recordedThrow.calcPoint();
        }
        value = total;
    }

    public int value() {
        return value;
    }
}
