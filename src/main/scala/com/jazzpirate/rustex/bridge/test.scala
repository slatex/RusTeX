package com.jazzpirate.rustex.bridge
import java.util
import scala.collection.mutable

object Implicits {
  import scala.jdk.CollectionConverters._
  implicit def apply[A](ls : List[A]) : util.ArrayList[A] = new util.ArrayList(ls.asJava)
}
import Implicits._

private class Bridge {
  @native def test(ls : util.ArrayList[JExecutable]) : Boolean
}

class JInterpreter {
  private var pointer : Long = 0

  @native def jobname() : String
}
abstract class JExecutable(val name : String) {
  def execute(_int: JInterpreter) : Boolean
}


object Bridge {
  System.load("/home/jazzpirate/work/Software/RusTeX/rustexbridge/target/debug/librustex_java.so")
  //System.load("/home/jazzpirate/work/Software/RusTeX/librustex.so")
  private val bridge = new Bridge
  def test() = {
    bridge.test(new JExecutable("pdfoutput") {
      override def execute(_int: JInterpreter): Boolean = {
        println("Fuck yeah!" + _int.jobname())
        sys.exit()
        true
      }
    } :: Nil)
  }
}


object Test {
  def main(args: Array[String]): Unit = {
    Bridge.test()
  }
}