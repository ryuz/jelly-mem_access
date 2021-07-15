


#pragma once

#include <random>
#include "jelly/simulator/Manager.h"


namespace jelly {
namespace simulator {


template<typename CT, typename RT, typename DistTp = std::uniform_int_distribution<>, typename RandTp = std::default_random_engine>
class RandomNode : public Node
{
protected:
    CT*     m_signal_clk;
    RT*     m_signal_rand;
    DistTp  m_dist;
    RandTp  m_engine;

    bool    m_clk;

    RandomNode(CT* signal_clk, RT* signal_rand, DistTp dist, RandTp engine)
    {
        m_signal_clk  = signal_clk;
        m_signal_rand = signal_rand;
        m_dist        = dist;
        m_engine      = engine;
    }

public:
    static std::shared_ptr<RandomNode> Create(CT* signal_clk, RT* signal_rand, DistTp dist, RandTp engine=RandTp(1))
    {
        return std::shared_ptr<RandomNode>(new RandomNode(signal_clk, signal_rand, dist, engine));
    }

protected:
    sim_time_t Initialize(Manager* manager) override
    {
        *m_signal_rand = m_dist(m_engine);
        return 0;
    };

    void PreProc(Manager* manager) override
    {
        m_clk = (*m_signal_clk != 0);
    }

    void PostProc(Manager* manager) override
    {
        // リセット解除で posedge clk の時だけ処理
        if ( !(!m_clk && *m_signal_clk != 0) ) {
            return;
        }
        *m_signal_rand = m_dist(m_engine);
    }
};

template<typename CT, typename RT, typename DistTp = std::uniform_int_distribution<>, typename RandTp = std::default_random_engine>
std::shared_ptr< RandomNode<CT, RT, DistTp, RandTp> > RandomNode_Create(CT* signal_clk, RT* signal_rand, DistTp dist, RandTp engine=RandTp(1))
{
    return RandomNode<CT, RT, DistTp, RandTp>::Create(signal_clk, signal_rand, dist, engine);
}

}
}


// end of file
