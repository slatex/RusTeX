package com.jazzpirate.rustex.bridge
import java.util
import scala.collection.mutable

object Implicits {
  import scala.jdk.CollectionConverters._
  implicit def apply[A](ls : List[A]) : util.ArrayList[A] = new util.ArrayList(ls.asJava)
}
import Implicits._
private class Bridge {
  @native def initialize() : Boolean
}
/*

class JInterpreter {
  private var pointer : Long = 0

  @native def jobname() : String
}
abstract class JExecutable(val name : String) {
  def execute(_int: JInterpreter) : Boolean
}

 */


object Bridge {
  //System.load("/home/jazzpirate/work/Software/RusTeX/rustexbridge/target/debug/librustex_java.so")
  private var bridge : Option[Bridge] = None
  def initialize(path : String): Unit = {
    val syspath = System.getProperty("os.name").toUpperCase()

    val actualpath = path + "/" + {
      if (syspath.startsWith("WINDOWS")) "i686-pc-windows-gnu/release/rustex_java.dll"
      else if (syspath.startsWith("MAC")) "x86_64-apple-darwin/release/librustex_java.dylib"
      else "x86_64-unknown-linux-gnu/release/librustex_java.so"
    }
    System.load(actualpath)
    bridge = Some(new Bridge)
    bridge.get.initialize()
    println("TeX Engine Initialized")
  }
  //System.load("/home/jazzpirate/work/Software/RusTeX/librustex.so")
  //private val bridge = new Bridge
  //bridge.initialize()
  /*def test() = {
    bridge.test(new JExecutable("pdfoutput") {
      override def execute(_int: JInterpreter): Boolean = {
        println("Fuck yeah!" + _int.jobname())
        sys.exit()
        true
      }
    } :: Nil)
  }*/
}


object Test {
  def main(args: Array[String]): Unit = {
    Bridge.initialize("/Users/dennismuller/work/RusTeX/rustexbridge/target")
    //Bridge.initialize("/home/jazzpirate/work/Software/RusTeX/rustexbridge/target/x86_64-unknown-linux-gnu/release")
  }
}