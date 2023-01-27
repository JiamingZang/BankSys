# Banksystem
OS homework

22年6月初学rust时攒起来的伤心大作业

需求是银行系统，给定存储了账号和对应余额的csv文件

有大致以下几个要求：

- 要求使用索引通过账号查找余额：用网上找的b+tree代码实现，读取csv文件生成b+tree数据结构并存储为.btree格式的索引，由于这个需求比较简单，我直接将key对应为账号，value是余额。
- 转账，存取款，并能提示账号不存在：通过读取索引文件实现。
- 合并相同账号的余额：我是在生成索引时进行一个查找操作，就知道该账号是否已经存储过。
- 使用索引文件及内存缓冲等技术，使得转账操作尽可能快：对一个账号进行操作后我就把其信息存储到了内存中，下次查找时先在内存中查找，如果内存中没有，再查索引。

对于基本的存取款和转账操作还有几个需求

- 所有操作都是在并发状态下执行。即，同一账号可能同时被多个进程（或线程）操作：这个需求通过加锁即可实现。
- 当系统正在进行批处理操作时，发生实时操作，系统需要暂停该批处理操作，而先去处理实时操作，等到实时操作处理完毕，再继续执行该批处理操作：这里的存取款是实时操作，而发工资和发利息是对多个账号进行的批处理操作，对于批处理操作的暂停需求，我认为对线程简单暂停还是有些粗暴，因此引入了一个线程池，使用优先级队列将对账号进行操作的闭包传递给工作线程，实时操作操作一个账号，对应一个闭包，批处理操作操作多个账号，对应多个闭包，将实时操作的优先级设置的高一些，因此当批处理操作的多个闭包在队列中等待时，如果有实时操作进入，就会将实时操作的闭包放在队首，优先执行，通过实时操作的插队实现宏观上的批处理操作的暂停。

查找账号速度是瓶颈（上亿条），但是使用b+tree做索引，操作起来还是很快的。

线程池和b+tree、mpsc优先级队列都是github或者网络上的优秀代码，然后我攒起来写了个这玩意，感谢你们拯救我的大作业。

当然当时的心态是不求甚解拿来就用...

伤心是因为写的烂且分数低...唉但是我觉得思路真的能凑合看啊，而且效果就还算不错...

虽然是个控制台应用，虽然是攒的代码，但私以为总比一查查个几十秒还非要做个贼丑的前端强一点点吧，结果分还没人家高，累了。

心得体会就是要么不做要么做绝...
