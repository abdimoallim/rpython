class Counter:
  def __init__(self):
    self.count = 0
  def inc(self):
    self.count = self.count + 1
c = Counter()
c.inc()
c.count