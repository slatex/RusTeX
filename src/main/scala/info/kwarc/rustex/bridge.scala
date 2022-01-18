package info.kwarc.rustex

import java.util
import scala.collection.mutable

object Implicits {
  import scala.jdk.CollectionConverters._
  implicit def apply[A](ls : List[A]) : util.ArrayList[A] = new util.ArrayList(ls.asJava)
}
import Implicits._
private class Bridge {
  @native def initialize() : Boolean
  @native def parse(file:String,p:Params) : String//,memories:util.ArrayList[String]) : String
}
abstract class Params {
  var singlethreaded = false
  var do_log = false
  var store_in_file = false
  var copy_tokens_full = true
  var copy_commands_full = true
  def log(s:String)
  def write_16(s:String)
  def write_17(s:String)
  def write_18(s:String)
  def write_neg_1(s:String)
  def write_other(s:String)
  def message(s:String)
  def file_clopen(s:String)
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
    val actualpath = path + "/" + library_filename()
    System.load(actualpath)
    bridge = Some(new Bridge)
    bridge.get.initialize()
    //println("TeX Engine Initialized")
  }
  def parse(s : String,p:Params, memories:List[String] = Nil) = bridge match {
    case Some(b) => b.parse(s,p)//,Implicits(memories))
    case None => ???
  }
  def library_filename() = {
    val syspath = System.getProperty("os.name").toUpperCase()
    if (syspath.startsWith("WINDOWS")) "rustex_java.dll"
    else if (syspath.startsWith("MAC")) "librustex_java.dylib"
    else "librustex_java.so"
  }
  def initialized() = bridge.isDefined
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
    val testparams = new Params {
      override def log(s: String): Unit = {}//println("LOG: " + s)
      override def write_16(s: String): Unit = {}//println("WRITE16: " + s)
      override def write_17(s: String): Unit = {}//println("WRITE17: " + s)
      override def write_18(s: String): Unit = {}//println("WRITE18: " + s)
      override def write_neg_1(s: String): Unit = {}//println("WRITE-1: " + s)
      override def write_other(s: String): Unit = {}//println("OTHER: " + s)
      override def file_clopen(s: String): Unit = {}//println("FILE: " + s)
      override def message(s: String): Unit = {}//println("MSG: " + s)
    }
    //Bridge.initialize("/Users/dennismuller/work/RusTeX/rustexbridge/target")
    Bridge.initialize("/home/jazzpirate/work/Software/RusTeX/rustexbridge/target/x86_64-unknown-linux-gnu/release")
    println("jupyterNB.en.tex")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/IWGS/source/jupyterNB.en.tex",testparams,List("a","b","c"))
    println("BBPformula")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/BBPformula.en.tex",testparams,List("a","b","c"))
    println("alternatingharmonic")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/alternatingharmonicseries.en.tex",testparams,List("a","b","c"))
    println("asymptotic")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/asymptoticdensity.en.tex",testparams,List("a","b","c"))
    println("baxterhickerson")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/baxterhickersonfunction.en.tex",testparams,List("a","b","c"))
    println("chebyshev")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/chebyshevfunction.en.tex",testparams,List("a","b","c"))
    println("cosineintegralbig")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralbig.en.tex",testparams,List("a","b","c"))
    println("cosineintegralint")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralint.en.tex",testparams,List("a","b","c"))
    println("cosineintegralsmall")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralsmall.en.tex",testparams,List("a","b","c"))
    println("generalharmonic")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/generalharmonicseries.en.tex",testparams,List("a","b","c"))
    println("gregory number")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/gregorynumber.en.tex",testparams,List("a","b","c"))
    println("harmonic series")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/harmonicseries.en.tex",testparams,List("a","b","c"))
    println("hurwitzzeta")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/hurwitzzetafunction.en.tex",testparams,List("a","b","c"))
    println("hyperboliccosine")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/hyperboliccosineintegral.en.tex",testparams,List("a","b","c"))
    println("Done")
    //println(ret)
  }
}