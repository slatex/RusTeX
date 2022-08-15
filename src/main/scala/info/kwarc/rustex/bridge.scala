package info.kwarc.rustex

import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream

import java.io.{File, FileInputStream, FileOutputStream}
import java.net.URI
import java.util
import scala.collection.mutable

object Implicits {
  import scala.jdk.CollectionConverters._
  implicit def apply[A](ls : List[A]) : util.ArrayList[A] = new util.ArrayList(ls.asJava)
  implicit def apply[A](ls : util.List[A]) : List[A] = {
    ls.iterator().asScala.toList
  }
}
import Implicits._
private class Bridge {
  @native def initialize(path:String) : Boolean
  @native def parse(file:String,p:Params,memories:util.ArrayList[String]) : String
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
  def file_open(s:String)
  def file_close()
  private def error_i(msg:String,stacktrace:Array[Array[String]],files:Array[Array[String]]) = {
    error(msg,stacktrace.toList.map(s => (s(0),s(1))),files.toList.map(s => (s(0),s(1).toInt,s(2).toInt)))
  }
  def error(msg:String,stacktrace:List[(String,String)],files:List[(String,Int,Int)])
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
  def download(uri:String,filestr:String) : Unit = {
    val url = new URI(uri).toURL
    val conn = url.openConnection()
    val httpConn = conn.asInstanceOf[java.net.HttpURLConnection]
    val resp = httpConn.getResponseCode
    // setFollowRedirects does not actually follow redirects
    if (resp.toString.startsWith("30")) {
      val redirectURL = conn.getHeaderField("Location")
      download(redirectURL,filestr)
    }
    else if (!resp.toString.startsWith("40")) {
      val in = conn.getInputStream
      val file = new File(filestr)
      file.getParentFile.mkdirs()
      val out = new FileOutputStream(file)
      try {
        val bytearray = LazyList.continually(in.read).takeWhile(_ != -1).map(_.toByte).toArray
        out.write(bytearray)
      } finally {
        in.close()
        out.close()
      }
    }
  }
  //System.load("/home/jazzpirate/work/Software/RusTeX/rustexbridge/target/debug/librustex_java.so")
  private var bridge : Option[Bridge] = None
  def initialize(path : String): Unit = {
    val actualpath = path + "/" + library_filename()
    val file = new File(actualpath)
    if (!file.exists()) {
      download("https://github.com/slatex/RusTeX/releases/download/latest/" + library_filename(),actualpath)
    }
    System.load(actualpath)
    val pdffile = new File(path + "/" + library_filename("pdfium"))
    if (!pdffile.exists()) {
      val tgzname = {
        val syspath = System.getProperty("os.name").toUpperCase()
        "pdfium-" + (if (syspath.startsWith("WINDOWS")) "win"
        else if (syspath.startsWith("MAC")) "mac"
        else "linux") + "-x64.tgz"
      }
      val tgzpath = path + "/" + tgzname
      download("https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F5145/" + tgzname,tgzpath)
      val tgf = new File(tgzpath)
      val filein = new FileInputStream(tgf)
      val in = new TarArchiveInputStream(new GzipCompressorInputStream(filein))
      var next = in.getNextEntry
      var done = false
      try { while (next != null && !done) {
        if (next.getName.endsWith(library_filename("pdfium"))) {
          val out = new FileOutputStream(pdffile)
          val bytearray = LazyList.continually(in.read()).takeWhile(_ != -1).map(_.toByte).toArray
          out.write(bytearray)
          out.close()
          done = true
        } else next = in.getNextEntry
      } } finally {
        in.close()
        tgf.delete()
      }
    }
    val b = new Bridge
    b.initialize(path + "/")
    bridge = Some(b)
    //println("TeX Engine Initialized")
  }
  def parse(s : String,p:Params, memories:List[String] = Nil) = bridge match {
    case Some(b) => b.parse(s,p,Implicits(memories))
    case None => ???
  }
  def library_filename(libname:String = "rustex_java") = {
    val syspath = System.getProperty("os.name").toUpperCase()
    if (syspath.startsWith("WINDOWS")) libname + ".dll"
    else if (syspath.startsWith("MAC")) "lib" + libname + ".dylib"
    else "lib" + libname + ".so"
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
      override def log(s: String): Unit = println("LOG: " + s)
      override def write_16(s: String): Unit = println("WRITE16: " + s)
      override def write_17(s: String): Unit = println("WRITE17: " + s)
      override def write_18(s: String): Unit = println("WRITE18: " + s)
      override def write_neg_1(s: String): Unit = println("WRITE-1: " + s)
      override def write_other(s: String): Unit = println("OTHER: " + s)
      override def file_open(s: String): Unit = println("FILE OPEN: " + s)
      override def file_close(): Unit = println("FILE CLOSE")
      override def message(s: String): Unit = println("MSG: " + s)

      override def error(msg: String, stacktrace: List[(String, String)],files : List[(String,Int,Int)]): Unit =
        println("Error: " + msg + "\n\n" + stacktrace.map{case (a,b) => a + " - " + b}.mkString("\n") + "\n\n" +
          files.map{case (s,l,p) => s + "(" + l + "," + p + ")"}.mkString("\n")
        )
    }
    //Bridge.initialize("/Users/dennismuller/work/RusTeX/rustexbridge/target")
    Bridge.initialize("/home/jazzpirate/work/Software/RusTeX/rustexbridge/target/x86_64-unknown-linux-gnu/release")
    //val ret = Bridge.parse("/home/jazzpirate/work/LaTeX/Others/test.tex",testparams)
    println("jupyterNB.en.tex")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/IWGS/source/jupyterNB.en.tex",testparams,List("c_stex_module"))
    println("BBPformula")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/BBPformula.en.tex",testparams,List("c_stex_module"))
    println("alternatingharmonic")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/alternatingharmonicseries.en.tex",testparams,List("c_stex_module"))
    println("asymptotic")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/asymptoticdensity.en.tex",testparams,List("c_stex_module"))
    println("baxterhickerson")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/baxterhickersonfunction.en.tex",testparams,List("c_stex_module"))
    println("chebyshev")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/chebyshevfunction.en.tex",testparams,List("c_stex_module"))
    println("cosineintegralbig")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralbig.en.tex",testparams,List("c_stex_module"))
    println("cosineintegralint")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralint.en.tex",testparams,List("c_stex_module"))
    println("cosineintegralsmall")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralsmall.en.tex",testparams,List("c_stex_module"))
    println("generalharmonic")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/generalharmonicseries.en.tex",testparams,List("c_stex_module"))
    println("gregory number")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/gregorynumber.en.tex",testparams,List("c_stex_module"))
    println("harmonic series")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/harmonicseries.en.tex",testparams,List("c_stex_module"))
    println("hurwitzzeta")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/hurwitzzetafunction.en.tex",testparams,List("c_stex_module"))
    println("hyperboliccosine")
    Bridge.parse("/home/jazzpirate/work/MathHub/smglom/analysis/source/hyperboliccosineintegral.en.tex",testparams,List("c_stex_module"))
    println("Done")
    //println(ret)
  }
}